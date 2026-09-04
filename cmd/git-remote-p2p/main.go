package main

import (
	"fmt"
	"os"
	"strings"

	gitremote "github.com/ipfs/kubo/plugin/plugins/git-remote"
)

func main() {
	if len(os.Args) < 3 {
		fmt.Fprintln(os.Stderr, "usage: git-remote-p2p <remote> <url>")
		os.Exit(1)
	}

	url := os.Args[2]
	peerID, repoPath, err := parseURL(url)
	if err != nil {
		fmt.Fprintf(os.Stderr, "invalid URL: %v\n", err)
		os.Exit(1)
	}

	h := &gitremote.Helper{
		Name:   "p2p",
		URL:    url,
		Stdin:  os.Stdin,
		Stdout: os.Stdout,
		Stderr: os.Stderr,
		Capabilities: []string{
			gitremote.CapConnect,
			gitremote.CapOption,
		},
	}

	// TODO: implement libp2p stream proxy to git-upload-pack / git-receive-pack
	_ = peerID
	_ = repoPath

	fmt.Fprintf(os.Stderr, "git-remote-p2p: %s/%s - connect capability not yet implemented\n", peerID, repoPath)

	if err := h.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}
}

func parseURL(url string) (peerID, repoPath string, err error) {
	// p2p://<peer-id>/<repo-path>
	if !strings.HasPrefix(url, "p2p://") {
		return "", "", fmt.Errorf("unsupported URL format: %s", url)
	}
	rest := url[len("p2p://"):]
	parts := strings.SplitN(rest, "/", 2)
	if len(parts) < 1 {
		return "", "", fmt.Errorf("invalid p2p URL: %s", url)
	}
	peerID = parts[0]
	if len(parts) == 2 {
		repoPath = parts[1]
	}
	return peerID, repoPath, nil
}
