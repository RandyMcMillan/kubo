package main

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"strings"

	"github.com/ipfs/boxo/files"
	"github.com/ipfs/boxo/path"
	"github.com/ipfs/kubo/client/rpc"
	gitremote "github.com/ipfs/kubo/plugin/plugins/git-remote"
)

func main() {
	if len(os.Args) < 3 {
		fmt.Fprintln(os.Stderr, "usage: git-remote-ipfs <remote> <url>")
		os.Exit(1)
	}

	url := os.Args[2]
	p, err := parseURL(url)
	if err != nil {
		fmt.Fprintf(os.Stderr, "invalid URL: %v\n", err)
		os.Exit(1)
	}

	api, err := rpc.NewLocalApi()
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to connect to IPFS API: %v\n", err)
		os.Exit(1)
	}

	h := &gitremote.Helper{
		Name:   "ipfs",
		URL:    url,
		Stdin:  os.Stdin,
		Stdout: os.Stdout,
		Stderr: os.Stderr,
		Capabilities: []string{
			gitremote.CapFetch,
			gitremote.CapOption,
		},
	}

	refMap, err := loadRefs(api, p)
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to load refs: %v\n", err)
		os.Exit(1)
	}

	h.List = func(forPush bool) ([]gitremote.Ref, error) {
		var refs []gitremote.Ref
		for name, sha := range refMap {
			refs = append(refs, gitremote.Ref{Name: name, SHA: sha})
		}
		return refs, nil
	}

	h.Fetch = func(ref string) error {
		sha, ok := refMap[ref]
		if !ok {
			return fmt.Errorf("ref not found: %s", ref)
		}
		return fetchObjects(api, p, sha)
	}

	if err := h.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}
}

func parseURL(url string) (path.Path, error) {
	for _, pref := range []string{"ipfs://ipfs/", "ipfs:///ipfs/"} {
		if strings.HasPrefix(url, pref) {
			return path.NewPath("/ipfs/" + url[len(pref):])
		}
	}
	if strings.HasPrefix(url, "ipfs://") {
		return path.NewPath("/ipfs/" + url[len("ipfs://"):])
	}
	return nil, fmt.Errorf("unsupported URL format: %s", url)
}

func loadRefs(api *rpc.HttpApi, p path.Path) (map[string]string, error) {
	ctx := context.Background()
	refPath, err := path.NewPath(p.String() + "/refs.json")
	if err != nil {
		return nil, err
	}

	node, err := api.Unixfs().Get(ctx, refPath)
	if err != nil {
		// fallback: no refs manifest, return empty
		return map[string]string{}, nil
	}
	defer node.Close()

	f, ok := node.(files.File)
	if !ok {
		return nil, fmt.Errorf("refs.json is not a file")
	}

	data, err := io.ReadAll(f)
	if err != nil {
		return nil, err
	}

	var refs map[string]string
	if err := json.Unmarshal(data, &refs); err != nil {
		return nil, err
	}
	return refs, nil
}

func fetchObjects(api *rpc.HttpApi, p path.Path, sha string) error {
	fmt.Fprintf(os.Stderr, "fetch %s from %s not yet fully implemented\n", sha, p.String())
	return nil
}
