package nostr

import (
	"testing"
)

func TestParseConfigNil(t *testing.T) {
	cfg, err := parseConfig(nil)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if cfg.Enabled {
		t.Error("expected disabled by default")
	}
}

func TestParseConfigFull(t *testing.T) {
	input := map[string]any{
		"Enabled":           true,
		"Relays":            []any{"wss://relay.example.com", "wss://relay2.example.com"},
		"PrivateKey":        "deadbeef",
		"PublishRepoEvents": true,
		"PublishPeerEvents": true,
		"PublishFileEvents": true,
	}
	cfg, err := parseConfig(input)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !cfg.Enabled {
		t.Error("expected enabled")
	}
	if len(cfg.Relays) != 2 {
		t.Errorf("expected 2 relays, got %d", len(cfg.Relays))
	}
	if cfg.PrivateKey != "deadbeef" {
		t.Errorf("unexpected private key: %s", cfg.PrivateKey)
	}
	if !cfg.PublishRepoEvents {
		t.Error("expected PublishRepoEvents true")
	}
	if !cfg.PublishPeerEvents {
		t.Error("expected PublishPeerEvents true")
	}
	if !cfg.PublishFileEvents {
		t.Error("expected PublishFileEvents true")
	}
}

func TestParseConfigEmptyRelays(t *testing.T) {
	input := map[string]any{
		"Relays": []any{},
	}
	cfg, err := parseConfig(input)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(cfg.Relays) != 0 {
		t.Errorf("expected 0 relays, got %d", len(cfg.Relays))
	}
}

func TestParseConfigInvalidType(t *testing.T) {
	_, err := parseConfig("not a map")
	if err == nil {
		t.Error("expected error for invalid config type")
	}
}
