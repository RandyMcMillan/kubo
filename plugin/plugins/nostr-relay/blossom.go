package nostrrelay

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strconv"

	"github.com/nbd-wtf/go-nostr/nip94"
)

// handleBlossomUpload implements NIP-96 file upload backed by IPFS.
func (r *Relay) handleBlossomUpload(w http.ResponseWriter, req *http.Request) {
	if req.Method != http.MethodPost {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}

	// limit upload size to 100MB
	req.Body = http.MaxBytesReader(w, req.Body, 100*1024*1024)

	file, header, err := req.FormFile("file")
	if err != nil {
		http.Error(w, "missing file", http.StatusBadRequest)
		return
	}
	defer file.Close()

	data, err := io.ReadAll(file)
	if err != nil {
		http.Error(w, "failed to read file", http.StatusInternalServerError)
		return
	}

	// compute SHA-256 for NIP-94
	hash := sha256.Sum256(data)
	hashHex := hex.EncodeToString(hash[:])

	// add to IPFS
	if r.ipfsAPI == nil {
		http.Error(w, "IPFS backend unavailable", http.StatusServiceUnavailable)
		return
	}

	ctx := req.Context()
	c, err := r.ipfsAPI.Add(ctx, data)
	if err != nil {
		http.Error(w, "failed to store file", http.StatusInternalServerError)
		return
	}

	// build response with NIP-94 event
	url := fmt.Sprintf("ipfs://%s", c.String())
	resp := map[string]interface{}{
		"status":     "success",
		"message":    "upload successful",
		"nip94_event": buildNIP94Event(url, hashHex, header.Filename, int64(len(data))),
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(resp)
}

func buildNIP94Event(url, hashHex, filename string, size int64) map[string]interface{} {
	fm := nip94.FileMetadata{
		URL:   url,
		X:     hashHex,
		M:     http.DetectContentType([]byte(filename)), // simplistic
		Size:  strconv.FormatInt(size, 10),
	}

	return map[string]interface{}{
		"tags":    fm.ToTags(),
		"content": "",
	}
}
