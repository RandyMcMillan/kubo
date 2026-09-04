package main

import (
	"context"
	"fmt"
	"os"
	"os/signal"
	"syscall"

	nostrrelay "github.com/ipfs/kubo/plugin/plugins/nostr-relay"
)

func main() {
	addr := os.Getenv("NOSTR_RELAY_ADDR")
	if addr == "" {
		addr = "127.0.0.1:7777"
	}

	ipfs, err := nostrrelay.NewIPFSClient()
	if err != nil {
		fmt.Fprintf(os.Stderr, "warning: IPFS not available: %v\n", err)
		ipfs = nil
	}

	relay := nostrrelay.NewRelay(addr, ipfs)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, os.Interrupt, syscall.SIGTERM)
	go func() {
		<-sigCh
		fmt.Fprintln(os.Stderr, "shutting down...")
		cancel()
	}()

	fmt.Fprintf(os.Stderr, "nostr relay starting on %s\n", addr)
	if err := relay.Start(ctx); err != nil {
		fmt.Fprintf(os.Stderr, "relay error: %v\n", err)
		os.Exit(1)
	}
}
