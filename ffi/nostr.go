package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"context"
	"encoding/json"
	"fmt"
	"sync"
	"time"

	"github.com/nbd-wtf/go-nostr"
	"github.com/nbd-wtf/go-nostr/nip05"
	"github.com/nbd-wtf/go-nostr/nip19"
)

//export kubo_nostr_generate_key
func kubo_nostr_generate_key() *C.char {
	key := nostr.GeneratePrivateKey()
	if key == "" {
		setError(fmt.Errorf("failed to generate key"))
		return nil
	}
	setError(nil)
	return C.CString(key)
}

//export kubo_nostr_get_public_key
func kubo_nostr_get_public_key(sk *C.char) *C.char {
	pk, err := nostr.GetPublicKey(C.GoString(sk))
	if err != nil {
		setError(fmt.Errorf("get public key: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(pk)
}

//export kubo_nostr_event_sign
func kubo_nostr_event_sign(sk *C.char, content *C.char, kind C.int) *C.char {
	evt := nostr.Event{
		CreatedAt: nostr.Now(),
		Kind:      int(kind),
		Content:   C.GoString(content),
		Tags:      make(nostr.Tags, 0),
	}

	if err := evt.Sign(C.GoString(sk)); err != nil {
		setError(fmt.Errorf("sign event: %w", err))
		return nil
	}

	jsonBytes, err := json.Marshal(evt)
	if err != nil {
		setError(fmt.Errorf("marshal event: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(string(jsonBytes))
}

//export kubo_nostr_event_verify
func kubo_nostr_event_verify(jsonStr *C.char) int64 {
	var evt nostr.Event
	if err := json.Unmarshal([]byte(C.GoString(jsonStr)), &evt); err != nil {
		setError(fmt.Errorf("unmarshal event: %w", err))
		return -1
	}

	if ok, err := evt.CheckSignature(); err != nil || !ok {
		setError(fmt.Errorf("invalid signature"))
		return 0
	}

	setError(nil)
	return 1
}

//export kubo_nostr_nip19_encode_pubkey
func kubo_nostr_nip19_encode_pubkey(hex *C.char) *C.char {
	npub, err := nip19.EncodePublicKey(C.GoString(hex))
	if err != nil {
		setError(fmt.Errorf("nip19 encode pubkey: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(npub)
}

//export kubo_nostr_nip19_decode_pubkey
func kubo_nostr_nip19_decode_pubkey(bech32 *C.char) *C.char {
	prefix, data, err := nip19.Decode(C.GoString(bech32))
	if err != nil {
		setError(fmt.Errorf("nip19 decode: %w", err))
		return nil
	}
	if prefix != "npub" {
		setError(fmt.Errorf("expected npub prefix, got %s", prefix))
		return nil
	}
	pk, ok := data.(string)
	if !ok {
		setError(fmt.Errorf("invalid npub data"))
		return nil
	}
	setError(nil)
	return C.CString(pk)
}

//export kubo_nostr_nip19_encode_seckey
func kubo_nostr_nip19_encode_seckey(hex *C.char) *C.char {
	nsec, err := nip19.EncodePrivateKey(C.GoString(hex))
	if err != nil {
		setError(fmt.Errorf("nip19 encode seckey: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(nsec)
}

//export kubo_nostr_nip19_decode_seckey
func kubo_nostr_nip19_decode_seckey(bech32 *C.char) *C.char {
	prefix, data, err := nip19.Decode(C.GoString(bech32))
	if err != nil {
		setError(fmt.Errorf("nip19 decode: %w", err))
		return nil
	}
	if prefix != "nsec" {
		setError(fmt.Errorf("expected nsec prefix, got %s", prefix))
		return nil
	}
	sk, ok := data.(string)
	if !ok {
		setError(fmt.Errorf("invalid nsec data"))
		return nil
	}
	setError(nil)
	return C.CString(sk)
}

//export kubo_nostr_nip19_encode_note
func kubo_nostr_nip19_encode_note(hex *C.char) *C.char {
	note, err := nip19.EncodeNote(C.GoString(hex))
	if err != nil {
		setError(fmt.Errorf("nip19 encode note: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(note)
}

//export kubo_nostr_nip19_decode_note
func kubo_nostr_nip19_decode_note(bech32 *C.char) *C.char {
	prefix, data, err := nip19.Decode(C.GoString(bech32))
	if err != nil {
		setError(fmt.Errorf("nip19 decode: %w", err))
		return nil
	}
	if prefix != "note" {
		setError(fmt.Errorf("expected note prefix, got %s", prefix))
		return nil
	}
	id, ok := data.(string)
	if !ok {
		setError(fmt.Errorf("invalid note data"))
		return nil
	}
	setError(nil)
	return C.CString(id)
}

//export kubo_nostr_nip19_encode_entity
func kubo_nostr_nip19_encode_entity(pubkey *C.char, kind C.int, identifier *C.char, relays *C.char) *C.char {
	var relayList []string
	if r := C.GoString(relays); r != "" {
		relayList = append(relayList, r)
	}
	naddr, err := nip19.EncodeEntity(C.GoString(pubkey), int(kind), C.GoString(identifier), relayList)
	if err != nil {
		setError(fmt.Errorf("nip19 encode entity: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(naddr)
}

//export kubo_nostr_nip19_decode_entity
func kubo_nostr_nip19_decode_entity(bech32 *C.char) *C.char {
	prefix, data, err := nip19.Decode(C.GoString(bech32))
	if err != nil {
		setError(fmt.Errorf("nip19 decode: %w", err))
		return nil
	}
	if prefix != "naddr" {
		setError(fmt.Errorf("expected naddr prefix, got %s", prefix))
		return nil
	}
	entity, ok := data.(nostr.EntityPointer)
	if !ok {
		setError(fmt.Errorf("invalid naddr data"))
		return nil
	}
	result, err := json.Marshal(entity)
	if err != nil {
		setError(fmt.Errorf("marshal entity: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(result))
}

//export kubo_nostr_nip05_verify
func kubo_nostr_nip05_verify(identifier *C.char, pubkey *C.char) int64 {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	profile, err := nip05.QueryIdentifier(ctx, C.GoString(identifier))
	if err != nil {
		setError(fmt.Errorf("nip05 query: %w", err))
		return -1
	}

	if profile.PublicKey != C.GoString(pubkey) {
		setError(fmt.Errorf("nip05 pubkey mismatch"))
		return 0
	}

	setError(nil)
	return 1
}

//export kubo_nostr_nip05_query
func kubo_nostr_nip05_query(identifier *C.char) *C.char {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	profile, err := nip05.QueryIdentifier(ctx, C.GoString(identifier))
	if err != nil {
		setError(fmt.Errorf("nip05 query: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(profile.PublicKey)
}

var (
	nostrRelaysMu    sync.RWMutex
	nostrRelays      = make(map[uint64]*nostr.Relay)
	nostrRelayNext   uint64 = 1
)

//export kubo_nostr_relay_connect
func kubo_nostr_relay_connect(url *C.char) uint64 {
	ctx, cancel := context.WithCancel(context.Background())
	relay := nostr.NewRelay(ctx, C.GoString(url))

	connectCtx, connectCancel := context.WithTimeout(ctx, 7*time.Second)
	defer connectCancel()

	if err := relay.Connect(connectCtx); err != nil {
		cancel()
		setError(fmt.Errorf("relay connect: %w", err))
		return 0
	}

	nostrRelaysMu.Lock()
	handle := nostrRelayNext
	nostrRelayNext++
	nostrRelays[handle] = relay
	nostrRelaysMu.Unlock()

	setError(nil)
	return handle
}

//export kubo_nostr_relay_close
func kubo_nostr_relay_close(handle uint64) int64 {
	nostrRelaysMu.Lock()
	relay, ok := nostrRelays[handle]
	if ok {
		delete(nostrRelays, handle)
	}
	nostrRelaysMu.Unlock()

	if !ok {
		setError(fmt.Errorf("invalid relay handle %d", handle))
		return -1
	}

	relay.Close()
	setError(nil)
	return 0
}

//export kubo_nostr_relay_publish
func kubo_nostr_relay_publish(handle uint64, eventJSON *C.char) int64 {
	nostrRelaysMu.RLock()
	relay, ok := nostrRelays[handle]
	nostrRelaysMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid relay handle %d", handle))
		return -1
	}

	var event nostr.Event
	if err := json.Unmarshal([]byte(C.GoString(eventJSON)), &event); err != nil {
		setError(fmt.Errorf("unmarshal event: %w", err))
		return -1
	}

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	if err := relay.Publish(ctx, event); err != nil {
		setError(fmt.Errorf("relay publish: %w", err))
		return -1
	}

	setError(nil)
	return 0
}
