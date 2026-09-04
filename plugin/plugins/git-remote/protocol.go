package gitremote

import (
	"bufio"
	"fmt"
	"io"
	"strings"
)

// Capabilities that a remote helper can support.
const (
	CapFetch    = "fetch"
	CapPush     = "push"
	CapOption   = "option"
	CapRefspec  = "refspec"
	CapConnect  = "connect"
	CapImport   = "import"
	CapExport   = "export"
	CapGet      = "get"
)

// Helper implements the git remote-helper protocol.
type Helper struct {
	Name   string
	URL    string
	Stdin  io.Reader
	Stdout io.Writer
	Stderr io.Writer

	Capabilities []string
	Refspecs     []string

	// List returns the remote refs. If forPush is true, the helper may
	// return a different set of refs (e.g., excluding tags).
	List func(forPush bool) ([]Ref, error)

	// Fetch downloads the given ref and its objects.
	Fetch func(ref string) error

	// Push updates the remote ref with the local ref.
	Push func(localRef, remoteRef string) error
}

// Ref represents a remote ref.
type Ref struct {
	Name   string
	SHA    string
	Symbol string // for symrefs
}

// Run starts the command loop.
func (h *Helper) Run() error {
	scanner := bufio.NewScanner(h.Stdin)
	for scanner.Scan() {
		line := scanner.Text()
		if line == "" {
			continue
		}
		if err := h.handle(line); err != nil {
			return err
		}
	}
	return scanner.Err()
}

func (h *Helper) handle(line string) error {
	fields := strings.Fields(line)
	if len(fields) == 0 {
		return nil
	}
	cmd := fields[0]

	switch cmd {
	case "capabilities":
		return h.cmdCapabilities()
	case "list":
		forPush := len(fields) > 1 && fields[1] == "for-push"
		return h.cmdList(forPush)
	case "fetch":
		if len(fields) < 2 {
			return h.err("fetch requires a ref argument")
		}
		return h.cmdFetch(fields[1])
	case "push":
		if len(fields) < 2 {
			return h.err("push requires a refspec argument")
		}
		return h.cmdPush(fields[1])
	case "option":
		return h.cmdOption(fields[1:])
	default:
		return h.err("unknown command: " + cmd)
	}
}

func (h *Helper) cmdCapabilities() error {
	for _, c := range h.Capabilities {
		if _, err := fmt.Fprintln(h.Stdout, c); err != nil {
			return err
		}
	}
	for _, r := range h.Refspecs {
		if _, err := fmt.Fprintf(h.Stdout, "refspec %s\n", r); err != nil {
			return err
		}
	}
	_, err := fmt.Fprintln(h.Stdout)
	return err
}

func (h *Helper) cmdList(forPush bool) error {
	if h.List == nil {
		return h.err("list not supported")
	}
	refs, err := h.List(forPush)
	if err != nil {
		return h.err("list failed: " + err.Error())
	}
	for _, r := range refs {
		if r.Symbol != "" {
			_, err = fmt.Fprintf(h.Stdout, "@%s %s\n", r.Name, r.Symbol)
		} else {
			_, err = fmt.Fprintf(h.Stdout, "%s %s\n", r.SHA, r.Name)
		}
		if err != nil {
			return err
		}
	}
	_, err = fmt.Fprintln(h.Stdout)
	return err
}

func (h *Helper) cmdFetch(ref string) error {
	if h.Fetch == nil {
		return h.err("fetch not supported")
	}
	if err := h.Fetch(ref); err != nil {
		return h.err("fetch failed: " + err.Error())
	}
	_, err := fmt.Fprintln(h.Stdout)
	return err
}

func (h *Helper) cmdPush(refspec string) error {
	if h.Push == nil {
		return h.err("push not supported")
	}
	parts := strings.Split(refspec, ":")
	if len(parts) != 2 {
		return h.err("invalid refspec: " + refspec)
	}
	if err := h.Push(parts[0], parts[1]); err != nil {
		return h.err("push failed: " + err.Error())
	}
	_, err := fmt.Fprintln(h.Stdout, "ok", parts[1])
	return err
}

func (h *Helper) cmdOption(args []string) error {
	// unsupported options are ignored
	_, err := fmt.Fprintln(h.Stdout, "unsupported")
	return err
}

func (h *Helper) err(msg string) error {
	_, err := fmt.Fprintf(h.Stderr, "error: %s\n", msg)
	return err
}
