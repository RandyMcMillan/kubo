package nostrrelay

import (
	"context"
	"encoding/json"
	"net/http"
	"sync"
	"time"

	"github.com/coder/websocket"
	"github.com/ipfs/go-cid"
	gnostr "github.com/nbd-wtf/go-nostr"
)

// Relay is a hybrid nostr relay backed by IPFS.
type Relay struct {
	addr     string
	ipfsAPI  IPFSAdder

	mu       sync.RWMutex
	events   map[string]*gnostr.Event
	subs     map[string]*subscription
	httpSrv  *http.Server
}

type IPFSAdder interface {
	Add(ctx context.Context, data []byte) (cid.Cid, error)
	Cat(ctx context.Context, c cid.Cid) ([]byte, error)
}

type subscription struct {
	id      string
	filters gnostr.Filters
	conn    *websocket.Conn
	ctx     context.Context
	cancel  context.CancelFunc
}

// NewRelay creates a new relay on the given address.
func NewRelay(addr string, ipfs IPFSAdder) *Relay {
	return &Relay{
		addr:   addr,
		ipfsAPI: ipfs,
		events: make(map[string]*gnostr.Event),
		subs:   make(map[string]*subscription),
	}
}

// Start starts the HTTP and websocket server.
func (r *Relay) Start(ctx context.Context) error {
	mux := http.NewServeMux()
	mux.HandleFunc("/", r.handleWS)
	mux.HandleFunc("/upload", r.handleBlossomUpload)

	r.httpSrv = &http.Server{
		Addr:    r.addr,
		Handler: mux,
	}

	go func() {
		<-ctx.Done()
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		r.httpSrv.Shutdown(shutdownCtx)
	}()

	return r.httpSrv.ListenAndServe()
}

func (r *Relay) handleWS(w http.ResponseWriter, req *http.Request) {
	conn, err := websocket.Accept(w, req, &websocket.AcceptOptions{
		InsecureSkipVerify: true,
	})
	if err != nil {
		return
	}
	defer conn.Close(websocket.StatusNormalClosure, "")

	ctx := req.Context()
	for {
		_, data, err := conn.Read(ctx)
		if err != nil {
			return
		}

		var envelope []json.RawMessage
		if err := json.Unmarshal(data, &envelope); err != nil {
			continue
		}
		if len(envelope) == 0 {
			continue
		}

		var label string
		if err := json.Unmarshal(envelope[0], &label); err != nil {
			continue
		}

		switch label {
		case "EVENT":
			r.handleEvent(ctx, conn, envelope)
		case "REQ":
			r.handleReq(ctx, conn, envelope)
		case "CLOSE":
			r.handleClose(ctx, conn, envelope)
		}
	}
}

func (r *Relay) handleEvent(ctx context.Context, conn *websocket.Conn, env []json.RawMessage) {
	if len(env) < 2 {
		return
	}
	var evt gnostr.Event
	if err := json.Unmarshal(env[1], &evt); err != nil {
		return
	}
	if ok, _ := evt.CheckSignature(); !ok {
		r.writeNotice(conn, "invalid: signature verification failed")
		return
	}

	r.mu.Lock()
	r.events[evt.ID] = &evt
	r.mu.Unlock()

	// persist to IPFS asynchronously
	if r.ipfsAPI != nil {
		go r.persistEvent(ctx, &evt)
	}

	// broadcast to matching subscriptions
	r.broadcast(&evt)

	r.writeOK(conn, evt.ID, true, "")
}

func (r *Relay) handleReq(ctx context.Context, conn *websocket.Conn, env []json.RawMessage) {
	if len(env) < 3 {
		return
	}
	var subID string
	if err := json.Unmarshal(env[1], &subID); err != nil {
		return
	}

	var filters gnostr.Filters
	for i := 2; i < len(env); i++ {
		var f gnostr.Filter
		if err := json.Unmarshal(env[i], &f); err != nil {
			continue
		}
		filters = append(filters, f)
	}

	subCtx, cancel := context.WithCancel(ctx)
	sub := &subscription{
		id:      subID,
		filters: filters,
		conn:    conn,
		ctx:     subCtx,
		cancel:  cancel,
	}

	r.mu.Lock()
	r.subs[subID] = sub
	r.mu.Unlock()

	// send matching events
	r.mu.RLock()
	for _, evt := range r.events {
		if filters.Match(evt) {
			r.writeEvent(conn, subID, evt)
		}
	}
	r.mu.RUnlock()

	r.writeEOSE(conn, subID)
}

func (r *Relay) handleClose(ctx context.Context, conn *websocket.Conn, env []json.RawMessage) {
	if len(env) < 2 {
		return
	}
	var subID string
	if err := json.Unmarshal(env[1], &subID); err != nil {
		return
	}
	r.mu.Lock()
	if sub, ok := r.subs[subID]; ok {
		sub.cancel()
		delete(r.subs, subID)
	}
	r.mu.Unlock()
}

func (r *Relay) broadcast(evt *gnostr.Event) {
	r.mu.RLock()
	subs := make([]*subscription, 0, len(r.subs))
	for _, s := range r.subs {
		subs = append(subs, s)
	}
	r.mu.RUnlock()

	for _, sub := range subs {
		if sub.filters.Match(evt) {
			r.writeEvent(sub.conn, sub.id, evt)
		}
	}
}

func (r *Relay) persistEvent(ctx context.Context, evt *gnostr.Event) {
	data, err := json.Marshal(evt)
	if err != nil {
		return
	}
	_, err = r.ipfsAPI.Add(ctx, data)
	if err != nil {
		// log persistence failure
	}
}

func (r *Relay) writeEvent(conn *websocket.Conn, subID string, evt *gnostr.Event) {
	msg := []interface{}{"EVENT", subID, evt}
	r.writeJSON(conn, msg)
}

func (r *Relay) writeEOSE(conn *websocket.Conn, subID string) {
	r.writeJSON(conn, []interface{}{"EOSE", subID})
}

func (r *Relay) writeOK(conn *websocket.Conn, eventID string, ok bool, reason string) {
	r.writeJSON(conn, []interface{}{"OK", eventID, ok, reason})
}

func (r *Relay) writeNotice(conn *websocket.Conn, msg string) {
	r.writeJSON(conn, []interface{}{"NOTICE", msg})
}

func (r *Relay) writeJSON(conn *websocket.Conn, v interface{}) {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	data, _ := json.Marshal(v)
	conn.Write(ctx, websocket.MessageText, data)
}
