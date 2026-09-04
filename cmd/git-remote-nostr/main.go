package main

import (
	"context"
	"fmt"
	"os"
	"strings"

	gnostr "github.com/nbd-wtf/go-nostr"
	"github.com/nbd-wtf/go-nostr/nip34"
	gitremote "github.com/ipfs/kubo/plugin/plugins/git-remote"
)

func main() {
	if len(os.Args) < 3 {
		fmt.Fprintln(os.Stderr, "usage: git-remote-nostr <remote> <url>")
		os.Exit(1)
	}

	url := os.Args[2]
	relay, naddr, err := parseURL(url)
	if err != nil {
		fmt.Fprintf(os.Stderr, "invalid URL: %v\n", err)
		os.Exit(1)
	}

	ctx := context.Background()
	pool := gnostr.NewSimplePool(ctx)
	defer pool.Close("done")

	refs, cloneURLs, err := discoverRepo(ctx, pool, relay, naddr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to discover repo: %v\n", err)
		os.Exit(1)
	}

	h := &gitremote.Helper{
		Name:   "nostr",
		URL:    url,
		Stdin:  os.Stdin,
		Stdout: os.Stdout,
		Stderr: os.Stderr,
		Capabilities: []string{
			gitremote.CapFetch,
			gitremote.CapOption,
		},
	}

	h.List = func(forPush bool) ([]gitremote.Ref, error) {
		var result []gitremote.Ref
		for name, sha := range refs {
			result = append(result, gitremote.Ref{Name: name, SHA: sha})
		}
		return result, nil
	}

	h.Fetch = func(ref string) error {
		sha, ok := refs[ref]
		if !ok {
			return fmt.Errorf("ref not found: %s", ref)
		}
		if len(cloneURLs) > 0 {
			fmt.Fprintf(os.Stderr, "fetch %s from %s (clone URLs: %v)\n", sha, ref, cloneURLs)
		}
		return nil
	}

	if err := h.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}
}

func parseURL(url string) (relay, naddr string, err error) {
	// nostr://<relay>/<naddr>
	if !strings.HasPrefix(url, "nostr://") {
		return "", "", fmt.Errorf("unsupported URL format: %s", url)
	}
	rest := url[len("nostr://"):]
	parts := strings.SplitN(rest, "/", 2)
	if len(parts) != 2 {
		return "", "", fmt.Errorf("invalid nostr URL: %s", url)
	}
	relay = "wss://" + parts[0]
	naddr = parts[1]
	return relay, naddr, nil
}

func discoverRepo(ctx context.Context, pool *gnostr.SimplePool, relayURL, naddr string) (map[string]string, []string, error) {
	// For MVP, naddr is treated as a simple "pubkey:identifier" pair.
	parts := strings.SplitN(naddr, ":", 2)
	if len(parts) != 2 {
		return nil, nil, fmt.Errorf("invalid naddr format: %s", naddr)
	}
	pubkey, identifier := parts[0], parts[1]

	// Connect to relay
	r, err := pool.EnsureRelay(relayURL)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to connect to relay: %w", err)
	}

	// Fetch repository announcement
	filter := gnostr.Filter{
		Kinds:   []int{gnostr.KindRepositoryAnnouncement},
		Authors: []string{pubkey},
		Tags:    gnostr.TagMap{"d": []string{identifier}},
	}

	res, err := r.QuerySync(ctx, filter)
	if err != nil {
		return nil, nil, err
	}

	var repo nip34.Repository
	for _, evt := range res {
		repo = nip34.ParseRepository(*evt)
		break
	}

	if repo.ID == "" {
		return map[string]string{}, nil, nil
	}

	// Fetch repository state
	state := repo.FetchState(ctx, r)
	refs := make(map[string]string)
	if state != nil {
		refs["HEAD"] = state.HEAD
		for branch, commit := range state.Branches {
			refs["refs/heads/"+branch] = commit
		}
		for tag, commit := range state.Tags {
			refs["refs/tags/"+tag] = commit
		}
	}

	return refs, repo.Clone, nil
}
