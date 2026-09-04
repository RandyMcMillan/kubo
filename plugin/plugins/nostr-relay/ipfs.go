package nostrrelay

import (
	"context"
	"fmt"

	"github.com/ipfs/boxo/files"
	"github.com/ipfs/boxo/path"
	"github.com/ipfs/go-cid"
	"github.com/ipfs/kubo/client/rpc"
)

// IPFSClient wraps the Kubo RPC client to satisfy the IPFSAdder interface.
type IPFSClient struct {
	API *rpc.HttpApi
}

func NewIPFSClient() (*IPFSClient, error) {
	api, err := rpc.NewLocalApi()
	if err != nil {
		return nil, fmt.Errorf("failed to connect to IPFS API: %w", err)
	}
	return &IPFSClient{API: api}, nil
}

func (c *IPFSClient) Add(ctx context.Context, data []byte) (cid.Cid, error) {
	file := files.NewBytesFile(data)
	defer file.Close()

	p, err := c.API.Unixfs().Add(ctx, file)
	if err != nil {
		return cid.Undef, err
	}

	resolved, _, err := c.API.ResolvePath(ctx, p)
	if err != nil {
		return cid.Undef, err
	}
	return resolved.RootCid(), nil
}

func (c *IPFSClient) Cat(ctx context.Context, c2 cid.Cid) ([]byte, error) {
	p := path.FromCid(c2)
	node, err := c.API.Unixfs().Get(ctx, p)
	if err != nil {
		return nil, err
	}
	defer node.Close()

	f, ok := node.(files.File)
	if !ok {
		return nil, fmt.Errorf("not a file")
	}

	data := make([]byte, 0, 1024)
	buf := make([]byte, 1024)
	for {
		n, err := f.Read(buf)
		if n > 0 {
			data = append(data, buf[:n]...)
		}
		if err != nil {
			break
		}
	}
	return data, nil
}
