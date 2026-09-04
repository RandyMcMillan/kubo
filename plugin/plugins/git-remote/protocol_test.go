package gitremote

import (
	"bytes"
	"fmt"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestCapabilities(t *testing.T) {
	var out bytes.Buffer
	h := &Helper{
		Stdout:       &out,
		Capabilities: []string{"fetch", "option"},
		Refspecs:     []string{"refs/heads/*:refs/remotes/origin/*"},
	}

	require.NoError(t, h.handle("capabilities"))
	lines := strings.Split(strings.TrimSpace(out.String()), "\n")
	require.Contains(t, lines, "fetch")
	require.Contains(t, lines, "option")
	require.Contains(t, lines, "refspec refs/heads/*:refs/remotes/origin/*")
}

func TestList(t *testing.T) {
	var out bytes.Buffer
	h := &Helper{
		Stdout: &out,
		List: func(forPush bool) ([]Ref, error) {
			return []Ref{
				{Name: "refs/heads/main", SHA: "abc123"},
				{Name: "refs/tags/v1.0", SHA: "def456"},
			}, nil
		},
	}

	require.NoError(t, h.handle("list"))
	output := out.String()
	require.Contains(t, output, "abc123 refs/heads/main")
	require.Contains(t, output, "def456 refs/tags/v1.0")
}

func TestListForPush(t *testing.T) {
	var out bytes.Buffer
	called := false
	h := &Helper{
		Stdout: &out,
		List: func(forPush bool) ([]Ref, error) {
			called = true
			require.True(t, forPush)
			return []Ref{}, nil
		},
	}

	require.NoError(t, h.handle("list for-push"))
	require.True(t, called)
}

func TestFetch(t *testing.T) {
	var out bytes.Buffer
	var fetched string
	h := &Helper{
		Stdout: &out,
		Fetch: func(ref string) error {
			fetched = ref
			return nil
		},
	}

	require.NoError(t, h.handle("fetch refs/heads/main"))
	require.Equal(t, "refs/heads/main", fetched)
}

func TestPush(t *testing.T) {
	var out bytes.Buffer
	var local, remote string
	h := &Helper{
		Stdout: &out,
		Push: func(l, r string) error {
			local = l
			remote = r
			return nil
		},
	}

	require.NoError(t, h.handle("push refs/heads/main:refs/heads/main"))
	require.Equal(t, "refs/heads/main", local)
	require.Equal(t, "refs/heads/main", remote)
	require.Contains(t, out.String(), "ok refs/heads/main")
}

func TestPushInvalidRefspec(t *testing.T) {
	var stderr bytes.Buffer
	h := &Helper{
		Stderr: &stderr,
		Push:   func(_, _ string) error { return nil },
	}

	require.NoError(t, h.handle("push invalid"))
	require.Contains(t, stderr.String(), "invalid refspec")
}

func TestUnknownCommand(t *testing.T) {
	var stderr bytes.Buffer
	h := &Helper{
		Stderr: &stderr,
	}

	require.NoError(t, h.handle("frobnicate"))
	require.Contains(t, stderr.String(), "unknown command")
}

func TestRun(t *testing.T) {
	input := strings.NewReader("capabilities\nlist\n\n")
	var out bytes.Buffer
	h := &Helper{
		Stdin:        input,
		Stdout:       &out,
		Capabilities: []string{"fetch"},
		List: func(_ bool) ([]Ref, error) {
			return []Ref{{Name: "refs/heads/main", SHA: "abc123"}}, nil
		},
	}

	require.NoError(t, h.Run())
	output := out.String()
	require.Contains(t, output, "fetch")
	require.Contains(t, output, "abc123 refs/heads/main")
}

func TestRefSymbol(t *testing.T) {
	var out bytes.Buffer
	h := &Helper{
		Stdout: &out,
		List: func(_ bool) ([]Ref, error) {
			return []Ref{{Name: "HEAD", Symbol: "refs/heads/main"}}, nil
		},
	}

	require.NoError(t, h.handle("list"))
	require.Contains(t, out.String(), "@HEAD refs/heads/main")
}

func TestHelperErr(t *testing.T) {
	var stderr bytes.Buffer
	h := &Helper{Stderr: &stderr}
	_ = h.err("something broke")
	require.Contains(t, stderr.String(), "something broke")
}

func ExampleHelper_handle() {
	var out bytes.Buffer
	h := &Helper{
		Stdout:       &out,
		Capabilities: []string{"fetch"},
	}
	_ = h.handle("capabilities")
	fmt.Println(strings.TrimSpace(out.String()))
	// Output:
	// fetch
}
