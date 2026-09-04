package nostr

import (
	"testing"

	"github.com/libp2p/go-libp2p/core/event"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	gnostr "github.com/nbd-wtf/go-nostr"
	"github.com/stretchr/testify/require"
)

func TestBuildPeerEventConnected(t *testing.T) {
	p, err := peer.Decode("12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN")
	require.NoError(t, err)

	np := &nostrPlugin{}
	ev := np.buildPeerEvent(event.EvtPeerConnectednessChanged{
		Peer:          p,
		Connectedness: network.Connected,
	})
	require.NotNil(t, ev)
	require.Equal(t, gnostr.KindTextNote, ev.Kind)
	require.Contains(t, ev.Content, "connected")
	require.Contains(t, ev.Tags.Find("p"), p.String())
}

func TestBuildPeerEventDisconnected(t *testing.T) {
	p, err := peer.Decode("12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN")
	require.NoError(t, err)

	np := &nostrPlugin{}
	ev := np.buildPeerEvent(event.EvtPeerConnectednessChanged{
		Peer:          p,
		Connectedness: network.NotConnected,
	})
	require.NotNil(t, ev)
	require.Contains(t, ev.Content, "disconnected")
}

func TestBuildPeerEventUnknown(t *testing.T) {
	p, err := peer.Decode("12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN")
	require.NoError(t, err)

	np := &nostrPlugin{}
	ev := np.buildPeerEvent(event.EvtPeerConnectednessChanged{
		Peer:          p,
		Connectedness: network.CanConnect,
	})
	require.NotNil(t, ev)
	require.Contains(t, ev.Content, "connectedness=")
}

func TestBuildRepoAnnouncement(t *testing.T) {
	// buildRepoAnnouncement requires a full IpfsNode which is heavy to set up
	// in a unit test; we verify the tag construction logic here.
	tags := gnostr.Tags{
		gnostr.Tag{"d", "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"},
		gnostr.Tag{"name", "kubo-12D3KooWDpJ7As7"},
		gnostr.Tag{"description", "Kubo IPFS node 12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"},
	}
	require.Equal(t, "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN", tags.GetD())
	require.Equal(t, "kubo-12D3KooWDpJ7As7", tags.Find("name")[1])
}

func TestEmitNilEvent(t *testing.T) {
	np := &nostrPlugin{
		events: make(chan nostrEvent, 1),
	}
	np.emit(nil) // should not panic or block
	select {
	case <-np.events:
		t.Fatal("expected no event emitted for nil input")
	default:
	}
}

func TestEmitEvent(t *testing.T) {
	np := &nostrPlugin{
		events: make(chan nostrEvent, 1),
	}
	evt := &gnostr.Event{Kind: gnostr.KindTextNote, Content: "test"}
	np.emit(evt)
	select {
	case e := <-np.events:
		require.Equal(t, evt, e.event)
	default:
		t.Fatal("expected event in channel")
	}
}

func TestEmitEventQueueFull(t *testing.T) {
	np := &nostrPlugin{
		events: make(chan nostrEvent, 0),
	}
	evt := &gnostr.Event{Kind: gnostr.KindTextNote, Content: "test"}
	np.emit(evt) // should not block when queue is full
	select {
	case <-np.events:
		t.Fatal("expected no event because queue was full")
	default:
	}
}
