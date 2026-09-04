package nostr

import "fmt"

// Config holds the nostr bridge plugin configuration.
type Config struct {
	Enabled           bool     `json:"Enabled"`
	Relays            []string `json:"Relays"`
	PrivateKey        string   `json:"PrivateKey"`
	PublishRepoEvents bool     `json:"PublishRepoEvents"`
	PublishPeerEvents bool     `json:"PublishPeerEvents"`
	PublishFileEvents bool     `json:"PublishFileEvents"`
}

func parseConfig(config any) (Config, error) {
	var cfg Config
	if config == nil {
		return cfg, nil
	}

	m, ok := config.(map[string]any)
	if !ok {
		return cfg, fmt.Errorf("invalid config type %T", config)
	}

	if v, ok := m["Enabled"].(bool); ok {
		cfg.Enabled = v
	}
	if v, ok := m["Relays"].([]any); ok {
		cfg.Relays = make([]string, 0, len(v))
		for _, r := range v {
			if s, ok := r.(string); ok && s != "" {
				cfg.Relays = append(cfg.Relays, s)
			}
		}
	}
	if v, ok := m["PrivateKey"].(string); ok {
		cfg.PrivateKey = v
	}
	if v, ok := m["PublishRepoEvents"].(bool); ok {
		cfg.PublishRepoEvents = v
	}
	if v, ok := m["PublishPeerEvents"].(bool); ok {
		cfg.PublishPeerEvents = v
	}
	if v, ok := m["PublishFileEvents"].(bool); ok {
		cfg.PublishFileEvents = v
	}

	return cfg, nil
}
