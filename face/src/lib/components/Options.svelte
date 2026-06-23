<script lang="ts">
  import * as Select from "$lib/components/ui/select";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import Label from "$lib/components/ui/label/label.svelte";
  import { HugeiconsIcon } from "@hugeicons/svelte";
  import { HelpCircleIcon } from "@hugeicons/core-free-icons";
  import type { Aria2Options } from "$lib/aria2/types";

  export let open = false;
  export let draft: Partial<Aria2Options> = {};

  type TextField = {
    key: keyof Aria2Options;
    label: string;
    description: string;
    placeholder?: string;
  };

  type TextAreaField = {
    key: keyof Aria2Options;
    label: string;
    description: string;
    placeholder?: string;
    rows?: number;
  };

  type BooleanField = {
    key: keyof Aria2Options;
    label: string;
    description: string;
  };

  type SelectField = {
    key: keyof Aria2Options;
    label: string;
    description: string;
    options: readonly string[];
  };

  type RangeField = {
    key: keyof Aria2Options;
    label: string;
    description: string;
    min: number;
    max: number;
    step?: number;
    unit?: string;
    encode?: (value: number) => string;
    decode?: (value: string | undefined) => number;
    format?: (value: number) => string;
  };

  type OptionSection = {
    title: string;
    description: string;
    textFields?: TextField[];
    textAreaFields?: TextAreaField[];
    selectFields?: SelectField[];
    booleanFields?: BooleanField[];
    sliderFields?: RangeField[];
  };

  let workingDraft: Partial<Aria2Options> = {};
  let previousOpen = false;

  const cloneDraft = (value: Partial<Aria2Options>) =>
    JSON.parse(JSON.stringify(value ?? {})) as Partial<Aria2Options>;

  const generalTextFields: TextField[] = [
    {
      key: "dir",
      label: "dir",
      description: "Default directory for this download.",
      placeholder: "/downloads"
    },
    {
      key: "out",
      label: "out",
      description: "Output filename to use for the download.",
      placeholder: "file.iso"
    },
    {
      key: "checksum",
      label: "checksum",
      description: "Checksum expected by aria2, for example sha-1=....",
      placeholder: "sha-1=..."
    },
    {
      key: "gid",
      label: "gid",
      description: "Custom GID for the download task.",
      placeholder: "Custom GID"
    },
    {
      key: "referer",
      label: "referer",
      description: "HTTP referer header value.",
      placeholder: "https://example.com"
    },
    {
      key: "user-agent",
      label: "user-agent",
      description: "User agent string used for HTTP requests.",
      placeholder: "Mozilla/5.0 ..."
    },
    {
      key: "ssh-host-key-md",
      label: "ssh-host-key-md",
      description: "Expected SSH host key fingerprint.",
      placeholder: "md5 hash"
    },
    {
      key: "bt-external-ip",
      label: "bt-external-ip",
      description: "External IP address announced to BitTorrent peers.",
      placeholder: "203.0.113.10"
    },
    {
      key: "bt-exclude-tracker",
      label: "bt-exclude-tracker",
      description: "Trackers to exclude, comma separated.",
      placeholder: "tracker1,tracker2"
    },
    {
      key: "bt-prioritize-piece",
      label: "bt-prioritize-piece",
      description: "Prioritize specific pieces, for example head/tail ranges.",
      placeholder: "head=1M,tail=1M"
    },
    {
      key: "bt-request-peer-speed-limit",
      label: "bt-request-peer-speed-limit",
      description: "Requested peer upload/download speed limit.",
      placeholder: "50K"
    },
    {
      key: "bt-tracker",
      label: "bt-tracker",
      description: "Custom tracker URLs, comma separated.",
      placeholder: "udp://tracker.example:80/announce"
    },
    {
      key: "ftp-passwd",
      label: "ftp-passwd",
      description: "Password for FTP authentication.",
      placeholder: "FTP password"
    },
    {
      key: "ftp-proxy",
      label: "ftp-proxy",
      description: "Proxy used for FTP transfers.",
      placeholder: "http://proxy.example:8080"
    },
    {
      key: "ftp-proxy-passwd",
      label: "ftp-proxy-passwd",
      description: "Password for the FTP proxy.",
      placeholder: "FTP proxy password"
    },
    {
      key: "ftp-proxy-user",
      label: "ftp-proxy-user",
      description: "Username for the FTP proxy.",
      placeholder: "FTP proxy user"
    },
    {
      key: "ftp-user",
      label: "ftp-user",
      description: "Username for FTP authentication.",
      placeholder: "FTP user"
    },
    {
      key: "http-passwd",
      label: "http-passwd",
      description: "Password for HTTP authentication.",
      placeholder: "HTTP password"
    },
    {
      key: "http-proxy",
      label: "http-proxy",
      description: "Proxy used for HTTP transfers.",
      placeholder: "http://proxy.example:8080"
    },
    {
      key: "http-proxy-passwd",
      label: "http-proxy-passwd",
      description: "Password for the HTTP proxy.",
      placeholder: "HTTP proxy password"
    },
    {
      key: "http-proxy-user",
      label: "http-proxy-user",
      description: "Username for the HTTP proxy.",
      placeholder: "HTTP proxy user"
    },
    {
      key: "http-user",
      label: "http-user",
      description: "Username for HTTP authentication.",
      placeholder: "HTTP user"
    },
    {
      key: "https-proxy",
      label: "https-proxy",
      description: "Proxy used for HTTPS transfers.",
      placeholder: "http://proxy.example:8080"
    },
    {
      key: "https-proxy-passwd",
      label: "https-proxy-passwd",
      description: "Password for the HTTPS proxy.",
      placeholder: "HTTPS proxy password"
    },
    {
      key: "https-proxy-user",
      label: "https-proxy-user",
      description: "Username for the HTTPS proxy.",
      placeholder: "HTTPS proxy user"
    },
    {
      key: "all-proxy",
      label: "all-proxy",
      description: "Proxy used for all protocols.",
      placeholder: "http://proxy.example:8080"
    },
    {
      key: "all-proxy-passwd",
      label: "all-proxy-passwd",
      description: "Password for the global proxy.",
      placeholder: "Proxy password"
    },
    {
      key: "all-proxy-user",
      label: "all-proxy-user",
      description: "Username for the global proxy.",
      placeholder: "Proxy user"
    },
    {
      key: "no-proxy",
      label: "no-proxy",
      description: "Hosts or domains that should bypass proxy settings.",
      placeholder: "localhost,127.0.0.1"
    }
  ];

  const generalTextAreaFields: TextAreaField[] = [
    {
      key: "header",
      label: "header",
      description: "Custom headers, one per line.",
      rows: 4,
      placeholder: "Authorization: Bearer ..."
    },
    {
      key: "index-out",
      label: "index-out",
      description: "Output names by file index, one per line.",
      rows: 4,
      placeholder: "1\n2\n3"
    }
  ];

  const generalSelectFields: SelectField[] = [
    {
      key: "file-allocation",
      label: "file-allocation",
      description: "How files are allocated before the download begins.",
      options: ["none", "prealloc", "trunc", "falloc"]
    },
    {
      key: "ftp-type",
      label: "ftp-type",
      description: "Transfer type used for FTP downloads.",
      options: ["binary", "ascii"]
    },
    {
      key: "bt-min-crypto-level",
      label: "bt-min-crypto-level",
      description: "Minimum encryption level for BitTorrent connections.",
      options: ["plain", "arc4"]
    },
    {
      key: "follow-metalink",
      label: "follow-metalink",
      description: "How metalink sources should be followed.",
      options: ["true", "false", "mem"]
    },
    {
      key: "follow-torrent",
      label: "follow-torrent",
      description: "How torrent sources should be followed.",
      options: ["true", "false", "mem"]
    },
    {
      key: "metalink-preferred-protocol",
      label: "metalink-preferred-protocol",
      description: "Preferred protocol when multiple metalink mirrors exist.",
      options: ["http", "https", "ftp", "none"]
    },
    {
      key: "proxy-method",
      label: "proxy-method",
      description: "Method used when connecting through a proxy.",
      options: ["get", "tunnel"]
    },
    {
      key: "stream-piece-selector",
      label: "stream-piece-selector",
      description: "How streaming pieces are selected.",
      options: ["default", "inorder", "random", "geom"]
    },
    {
      key: "uri-selector",
      label: "uri-selector",
      description: "How aria2 selects the URI to try first.",
      options: ["inorder", "feedback", "adaptive"]
    }
  ];

  const generalBooleanFields: BooleanField[] = [
    { key: "allow-overwrite", label: "allow-overwrite", description: "Allow overwriting existing files." },
    {
      key: "allow-piece-length-change",
      label: "allow-piece-length-change",
      description: "Allow piece length changes during the download."
    },
    { key: "always-resume", label: "always-resume", description: "Always try to resume downloads." },
    { key: "async-dns", label: "async-dns", description: "Resolve hostnames asynchronously." },
    { key: "auto-file-renaming", label: "auto-file-renaming", description: "Rename duplicate files automatically." },
    { key: "bt-enable-hook-after-hash-check", label: "bt-enable-hook-after-hash-check", description: "Run hooks after hash checking." },
    { key: "bt-enable-lpd", label: "bt-enable-lpd", description: "Enable local peer discovery." },
    { key: "bt-force-encryption", label: "bt-force-encryption", description: "Require BitTorrent encryption." },
    { key: "bt-hash-check-seed", label: "bt-hash-check-seed", description: "Check seed hash before starting seeding." },
    { key: "bt-load-saved-metadata", label: "bt-load-saved-metadata", description: "Load saved torrent metadata if available." },
    { key: "bt-metadata-only", label: "bt-metadata-only", description: "Download metadata only." },
    { key: "bt-remove-unselected-file", label: "bt-remove-unselected-file", description: "Remove files that are not selected." },
    { key: "bt-require-crypto", label: "bt-require-crypto", description: "Reject torrents that do not use encryption." },
    { key: "bt-save-metadata", label: "bt-save-metadata", description: "Save torrent metadata after fetching it." },
    { key: "bt-seed-unverified", label: "bt-seed-unverified", description: "Seed even if the data is not verified." },
    { key: "check-integrity", label: "check-integrity", description: "Check file integrity after completion." },
    { key: "conditional-get", label: "conditional-get", description: "Use conditional HTTP requests." },
    { key: "content-disposition-default-utf8", label: "content-disposition-default-utf8", description: "Assume UTF-8 for Content-Disposition." },
    { key: "continue", label: "continue", description: "Continue partially downloaded files." },
    { key: "enable-http-keep-alive", label: "enable-http-keep-alive", description: "Keep HTTP connections alive." },
    { key: "enable-http-pipelining", label: "enable-http-pipelining", description: "Enable HTTP pipelining." },
    { key: "enable-mmap", label: "enable-mmap", description: "Use memory-mapped file I/O." },
    { key: "enable-peer-exchange", label: "enable-peer-exchange", description: "Enable peer exchange for torrents." },
    { key: "force-save", label: "force-save", description: "Force saving the file even if incomplete." },
    { key: "ftp-pasv", label: "ftp-pasv", description: "Use passive mode for FTP." },
    { key: "ftp-reuse-connection", label: "ftp-reuse-connection", description: "Reuse FTP connections when possible." },
    { key: "hash-check-only", label: "hash-check-only", description: "Only verify hashes without downloading." },
    { key: "http-accept-gzip", label: "http-accept-gzip", description: "Accept gzip-encoded HTTP responses." },
    { key: "http-auth-challenge", label: "http-auth-challenge", description: "Respond to HTTP authentication challenges." },
    { key: "http-no-cache", label: "http-no-cache", description: "Send no-cache headers on HTTP requests." },
    { key: "metalink-enable-unique-protocol", label: "metalink-enable-unique-protocol", description: "Use only unique protocols from metalinks." },
    { key: "no-netrc", label: "no-netrc", description: "Do not read credentials from .netrc." },
    { key: "realtime-chunk-checksum", label: "realtime-chunk-checksum", description: "Verify chunk checksums while downloading." },
    { key: "remote-time", label: "remote-time", description: "Preserve remote file timestamps." },
    { key: "remove-control-file", label: "remove-control-file", description: "Remove control files after completion." },
    { key: "reuse-uri", label: "reuse-uri", description: "Reuse already-tried URIs." },
    { key: "use-head", label: "use-head", description: "Use HEAD requests to probe remote files." }
  ];

  const bittorrentTextFields: TextField[] = [
    {
      key: "bt-tracker",
      label: "bt-tracker",
      description: "Tracker URLs to use for torrent downloads.",
      placeholder: "udp://tracker.example:80/announce"
    },
    {
      key: "bt-external-ip",
      label: "bt-external-ip",
      description: "External IP address reported to peers.",
      placeholder: "203.0.113.10"
    },
    {
      key: "bt-exclude-tracker",
      label: "bt-exclude-tracker",
      description: "Trackers to exclude, comma separated.",
      placeholder: "tracker1,tracker2"
    },
    {
      key: "bt-prioritize-piece",
      label: "bt-prioritize-piece",
      description: "Prioritize certain piece ranges.",
      placeholder: "head=1M,tail=1M"
    },
    {
      key: "bt-request-peer-speed-limit",
      label: "bt-request-peer-speed-limit",
      description: "Requested peer speed limit.",
      placeholder: "50K"
    }
  ];

  const bittorrentBooleanFields: BooleanField[] = [
    { key: "bt-enable-hook-after-hash-check", label: "bt-enable-hook-after-hash-check", description: "Run hooks after hash check completes." },
    { key: "bt-enable-lpd", label: "bt-enable-lpd", description: "Enable local peer discovery." },
    { key: "bt-force-encryption", label: "bt-force-encryption", description: "Require encryption for BitTorrent traffic." },
    { key: "bt-hash-check-seed", label: "bt-hash-check-seed", description: "Check seed hash before seeding." },
    { key: "bt-load-saved-metadata", label: "bt-load-saved-metadata", description: "Load metadata from saved state." },
    { key: "bt-metadata-only", label: "bt-metadata-only", description: "Download only BitTorrent metadata." },
    { key: "bt-remove-unselected-file", label: "bt-remove-unselected-file", description: "Remove unselected files from the torrent." },
    { key: "bt-require-crypto", label: "bt-require-crypto", description: "Reject torrents without encryption." },
    { key: "bt-save-metadata", label: "bt-save-metadata", description: "Save torrent metadata to disk." },
    { key: "bt-seed-unverified", label: "bt-seed-unverified", description: "Seed unverified data." }
  ];

  const bittorrentSliderFields: RangeField[] = [
    {
      key: "bt-max-peers",
      label: "bt-max-peers",
      description: "Maximum number of peers per torrent.",
      min: 1,
      max: 500,
      step: 1
    },
    {
      key: "bt-stop-timeout",
      label: "bt-stop-timeout",
      description: "Delay before stopping a torrent after completion.",
      min: 0,
      max: 60,
      step: 1
    },
    {
      key: "seed-ratio",
      label: "seed-ratio",
      description: "Stop seeding after reaching this ratio.",
      min: 0,
      max: 10,
      step: 0.1
    },
    {
      key: "seed-time",
      label: "seed-time",
      description: "Stop seeding after this many minutes.",
      min: 0,
      max: 60,
      step: 1
    },
    {
      key: "bt-tracker-connect-timeout",
      label: "bt-tracker-connect-timeout",
      description: "Timeout for connecting to trackers.",
      min: 0,
      max: 60,
      step: 1
    },
    {
      key: "bt-tracker-interval",
      label: "bt-tracker-interval",
      description: "Interval between tracker requests.",
      min: 0,
      max: 600,
      step: 5
    },
    {
      key: "bt-tracker-timeout",
      label: "bt-tracker-timeout",
      description: "Timeout for tracker responses.",
      min: 0,
      max: 600,
      step: 5
    }
  ];

  const networkSliderFields: RangeField[] = [
    {
      key: "connect-timeout",
      label: "connect-timeout",
      description: "Connection timeout in seconds.",
      min: 0,
      max: 120,
      step: 1
    },
    {
      key: "lowest-speed-limit",
      label: "lowest-speed-limit",
      description: "Minimum transfer speed in KiB/s.",
      min: 0,
      max: 10240,
      step: 10
    },
    {
      key: "max-connection-per-server",
      label: "max-connection-per-server",
      description: "Maximum parallel connections per server.",
      min: 1,
      max: 32,
      step: 1
    },
    {
      key: "max-download-limit",
      label: "max-download-limit",
      description: "Per-download maximum download speed. 0 means unlimited.",
      min: 0,
      max: 1000,
      step: 10,
      unit: "M",
      encode: (value) => (value === 0 ? "0" : `${value}M`),
      decode: (value) => {
        if (!value || value === "0") return 0;
        const parsed = Number(value.replace(/[^0-9.]/g, ""));
        return Number.isFinite(parsed) ? parsed : 0;
      },
      format: (value) => (value === 0 ? "Unlimited" : `${value} MiB`)
    },
    {
      key: "max-upload-limit",
      label: "max-upload-limit",
      description: "Per-download maximum upload speed. 0 means unlimited.",
      min: 0,
      max: 1000,
      step: 10,
      unit: "M",
      encode: (value) => (value === 0 ? "0" : `${value}M`),
      decode: (value) => {
        if (!value || value === "0") return 0;
        const parsed = Number(value.replace(/[^0-9.]/g, ""));
        return Number.isFinite(parsed) ? parsed : 0;
      },
      format: (value) => (value === 0 ? "Unlimited" : `${value} MiB`)
    },
    {
      key: "max-file-not-found",
      label: "max-file-not-found",
      description: "Allowed number of file-not-found errors.",
      min: 0,
      max: 20,
      step: 1
    },
    {
      key: "max-mmap-limit",
      label: "max-mmap-limit",
      description: "Maximum mmap usage in MiB.",
      min: 0,
      max: 2048,
      step: 64
    },
    {
      key: "max-resume-failure-tries",
      label: "max-resume-failure-tries",
      description: "Allowed resume failures before stopping.",
      min: 0,
      max: 20,
      step: 1
    },
    {
      key: "max-tries",
      label: "max-tries",
      description: "Maximum retry count.",
      min: 0,
      max: 100,
      step: 1
    },
    {
      key: "no-file-allocation-limit",
      label: "no-file-allocation-limit",
      description: "Allocation is skipped below this file size in MiB.",
      min: 0,
      max: 1024,
      step: 32
    },
    {
      key: "split",
      label: "split",
      description: "Number of connections per file.",
      min: 1,
      max: 32,
      step: 1
    },
    {
      key: "retry-wait",
      label: "retry-wait",
      description: "Delay between retry attempts in seconds.",
      min: 0,
      max: 60,
      step: 1
    }
  ];

  const mediaTextFields: TextField[] = [
    {
      key: "dir",
      label: "dir",
      description: "Default save directory for this item.",
      placeholder: "/downloads"
    },
    {
      key: "out",
      label: "out",
      description: "Output filename for this item.",
      placeholder: "file.iso"
    },
    {
      key: "referer",
      label: "referer",
      description: "Referer header for web downloads.",
      placeholder: "https://example.com"
    },
    {
      key: "user-agent",
      label: "user-agent",
      description: "User agent string for this item.",
      placeholder: "Mozilla/5.0 ..."
    }
  ];

  const mediaBooleanFields: BooleanField[] = [
    { key: "continue", label: "continue", description: "Continue partially downloaded data." },
    { key: "force-save", label: "force-save", description: "Force saving the file even if incomplete." },
    { key: "hash-check-only", label: "hash-check-only", description: "Only verify hashes." },
    { key: "no-netrc", label: "no-netrc", description: "Do not use credentials from .netrc." },
    { key: "pause-metadata", label: "pause-metadata", description: "Pause while fetching metadata." },
    { key: "remote-time", label: "remote-time", description: "Preserve the remote timestamp." },
    { key: "remove-control-file", label: "remove-control-file", description: "Remove control files after completion." },
    { key: "reuse-uri", label: "reuse-uri", description: "Reuse already-tried URIs." },
    { key: "use-head", label: "use-head", description: "Probe remote files with HEAD requests first." }
  ];

  const sections: OptionSection[] = [
    {
      title: "General",
      description: "Common local download options.",
      textFields: generalTextFields,
      textAreaFields: generalTextAreaFields,
      selectFields: generalSelectFields,
      booleanFields: generalBooleanFields
    },
    {
      title: "BitTorrent",
      description: "Torrent-specific local settings.",
      textFields: bittorrentTextFields,
      booleanFields: bittorrentBooleanFields,
      sliderFields: bittorrentSliderFields
    },
    {
      title: "HTTP / FTP / Proxy",
      description: "Network transport, proxy, and authentication settings.",
      sliderFields: networkSliderFields,
      booleanFields: [
        { key: "async-dns", label: "async-dns", description: "Resolve hostnames asynchronously." },
        { key: "conditional-get", label: "conditional-get", description: "Use conditional HTTP requests." },
        { key: "content-disposition-default-utf8", label: "content-disposition-default-utf8", description: "Assume UTF-8 for Content-Disposition." },
        { key: "enable-http-keep-alive", label: "enable-http-keep-alive", description: "Keep HTTP connections alive." },
        { key: "enable-http-pipelining", label: "enable-http-pipelining", description: "Enable HTTP pipelining." },
        { key: "ftp-pasv", label: "ftp-pasv", description: "Use passive mode for FTP." },
        { key: "ftp-reuse-connection", label: "ftp-reuse-connection", description: "Reuse FTP connections when possible." },
        { key: "http-accept-gzip", label: "http-accept-gzip", description: "Accept gzip-encoded HTTP responses." },
        { key: "http-auth-challenge", label: "http-auth-challenge", description: "Respond to HTTP auth challenges." },
        { key: "http-no-cache", label: "http-no-cache", description: "Send no-cache headers for HTTP requests." },
        { key: "no-netrc", label: "no-netrc", description: "Do not read credentials from .netrc." },
        { key: "enable-peer-exchange", label: "enable-peer-exchange", description: "Enable peer exchange for torrents." }
      ]
    },
    {
      title: "Metalink",
      description: "Mirror and metalink-related local settings.",
      textFields: [
        {
          key: "metalink-language",
          label: "metalink-language",
          description: "Preferred language in metalink metadata.",
          placeholder: "en"
        },
        {
          key: "metalink-location",
          label: "metalink-location",
          description: "Preferred location code in metalink metadata.",
          placeholder: "US"
        },
        {
          key: "metalink-os",
          label: "metalink-os",
          description: "Preferred operating system in metalink metadata.",
          placeholder: "linux"
        },
        {
          key: "metalink-version",
          label: "metalink-version",
          description: "Preferred version in metalink metadata.",
          placeholder: "1.0"
        }
      ],
      selectFields: [
        {
          key: "metalink-preferred-protocol",
          label: "metalink-preferred-protocol",
          description: "Preferred protocol when multiple mirrors are available.",
          options: ["http", "https", "ftp", "none"]
        }
      ],
      booleanFields: [
        {
          key: "metalink-enable-unique-protocol",
          label: "metalink-enable-unique-protocol",
          description: "Use only unique protocols from the metalink."
        }
      ]
    },
    {
      title: "Selection & Limits",
      description: "Piece selection, concurrency, and retry-related controls.",
      selectFields: [
        {
          key: "stream-piece-selector",
          label: "stream-piece-selector",
          description: "How stream pieces are selected.",
          options: ["default", "inorder", "random", "geom"]
        },
        {
          key: "uri-selector",
          label: "uri-selector",
          description: "How URIs are selected for each request.",
          options: ["inorder", "feedback", "adaptive"]
        },
        {
          key: "file-allocation",
          label: "file-allocation",
          description: "How files are allocated before downloading.",
          options: ["none", "prealloc", "trunc", "falloc"]
        },
        {
          key: "ftp-type",
          label: "ftp-type",
          description: "Transfer type used for FTP downloads.",
          options: ["binary", "ascii"]
        },
        {
          key: "bt-min-crypto-level",
          label: "bt-min-crypto-level",
          description: "Minimum encryption level for BitTorrent traffic.",
          options: ["plain", "arc4"]
        },
        {
          key: "follow-metalink",
          label: "follow-metalink",
          description: "How metalink inputs should be followed.",
          options: ["true", "false", "mem"]
        },
        {
          key: "follow-torrent",
          label: "follow-torrent",
          description: "How torrent inputs should be followed.",
          options: ["true", "false", "mem"]
        },
        {
          key: "proxy-method",
          label: "proxy-method",
          description: "Proxy method to use for HTTP/FTP proxy connections.",
          options: ["get", "tunnel"]
        }
      ]
    },
    {
      title: "Media / Misc",
      description: "Miscellaneous local controls used by common download types.",
      textFields: mediaTextFields,
      booleanFields: mediaBooleanFields
    }
  ];

  const getText = (key: keyof Aria2Options, fallback = "") => {
    const value = workingDraft[key];
    return typeof value === "string" ? value : fallback;
  };

  const setText = (key: keyof Aria2Options, value: string) => {
    workingDraft = { ...workingDraft, [key]: value };
  };

  const getTextArea = (key: keyof Aria2Options) => {
    const value = workingDraft[key];
    if (Array.isArray(value)) return value.join("\n");
    return typeof value === "string" ? value : "";
  };

  const setTextArea = (key: keyof Aria2Options, value: string) => {
    const lines = value
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean);

    workingDraft = { ...workingDraft, [key]: lines.length ? lines : undefined };
  };

  const getBoolean = (key: keyof Aria2Options) => getText(key) === "true";

  const setBoolean = (key: keyof Aria2Options, checked: boolean) => {
    setText(key, checked ? "true" : "false");
  };

  const getRangeValue = (field: RangeField) => {
    const raw = getText(field.key);
    if (field.decode) return field.decode(raw || undefined);
    const parsed = Number(raw);
    return Number.isFinite(parsed) ? parsed : field.min;
  };

  const setRangeValue = (field: RangeField, value: number) => {
    const encoded = field.encode ? field.encode(value) : String(value);
    setText(field.key, encoded);
  };

  const getSelectValue = (field: SelectField) => {
    const value = getText(field.key);
    return value || field.options[0];
  };

  const close = () => {
    open = false;
  };

  const save = () => {
    draft = cloneDraft(workingDraft);
    open = false;
  };

  $: if (open && !previousOpen) {
    workingDraft = cloneDraft(draft);
  }

  $: previousOpen = open;
</script>

{#if open}
  <Tooltip.Provider>
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div class="flex max-h-[90vh] w-full max-w-6xl flex-col rounded-2xl border border-sidebar-border/60 bg-background shadow-2xl">
        <div class="flex items-start justify-between gap-4 border-b border-sidebar-border/60 p-4">
          <div>
            <h2 class="text-lg font-semibold">Local Options</h2>
            <p class="text-sm text-muted-foreground">
              Edit aria2 local options for a single download task.
            </p>
          </div>

          <button
            type="button"
            onclick={close}
            class="rounded-md px-2 py-1 text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          >
            Close
          </button>
        </div>

        <div class="grid gap-4 overflow-y-auto p-4">
          {#each sections as section (section.title)}
            <section class="space-y-3 rounded-2xl border border-sidebar-border/60 bg-sidebar-background/30 p-4">
              <div>
                <h3 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">
                  {section.title}
                </h3>
                <p class="text-xs text-muted-foreground">{section.description}</p>
              </div>

              {#if section.textFields && section.textFields.length}
                <div class="grid gap-3 md:grid-cols-2">
                  {#each section.textFields as field (field.key)}
                    <div class="space-y-2 rounded-xl border border-sidebar-border/60 bg-sidebar-background/70 p-3">
                      <div class="flex items-center gap-1">
                        <Label class="block text-xs font-medium uppercase tracking-wide text-sidebar-foreground/70">
                          {field.label}
                        </Label>

                        <Tooltip.Root>
                          <Tooltip.Trigger>
                            <button
                              type="button"
                              class="inline-flex items-center text-muted-foreground transition-colors hover:text-foreground"
                              aria-label={`Help for ${field.label}`}
                            >
                              <HugeiconsIcon icon={HelpCircleIcon} strokeWidth={2} class="size-3.5" />
                            </button>
                          </Tooltip.Trigger>

                          <Tooltip.Content side="top" align="center" class="max-w-xs">
                            <p class="text-xs">{field.description}</p>
                          </Tooltip.Content>
                        </Tooltip.Root>
                      </div>

                      <input
                        type="text"
                        value={getText(field.key)}
                        placeholder={field.placeholder}
                        oninput={(event) => {
                          setText(field.key, (event.currentTarget as HTMLInputElement).value);
                        }}
                        class="w-full rounded-md border border-sidebar-border/60 bg-background px-3 py-2 text-sm outline-none ring-0 placeholder:text-muted-foreground/60"
                      />
                    </div>
                  {/each}
                </div>
              {/if}

              {#if section.textAreaFields && section.textAreaFields.length}
                <div class="grid gap-3">
                  {#each section.textAreaFields as field (field.key)}
                    <div class="space-y-2 rounded-xl border border-sidebar-border/60 bg-sidebar-background/70 p-3">
                      <div class="flex items-center gap-1">
                        <Label class="block text-xs font-medium uppercase tracking-wide text-sidebar-foreground/70">
                          {field.label}
                        </Label>

                        <Tooltip.Root>
                          <Tooltip.Trigger >
                            <button
                              type="button"
                              class="inline-flex items-center text-muted-foreground transition-colors hover:text-foreground"
                              aria-label={`Help for ${field.label}`}
                            >
                              <HugeiconsIcon icon={HelpCircleIcon} strokeWidth={2} class="size-3.5" />
                            </button>
                          </Tooltip.Trigger>

                          <Tooltip.Content side="top" align="center" class="max-w-xs">
                            <p class="text-xs">{field.description}</p>
                          </Tooltip.Content>
                        </Tooltip.Root>
                      </div>

                      <textarea
                        rows={field.rows ?? 4}
                        value={getTextArea(field.key)}
                        placeholder={field.placeholder}
                        oninput={(event) => {
                          setTextArea(field.key, (event.currentTarget as HTMLTextAreaElement).value);
                        }}
                        class="min-h-28 w-full rounded-md border border-sidebar-border/60 bg-background px-3 py-2 text-sm outline-none ring-0 placeholder:text-muted-foreground/60"
                      ></textarea>
                    </div>
                  {/each}
                </div>
              {/if}

              {#if section.selectFields && section.selectFields.length}
                <div class="grid gap-3 md:grid-cols-2">
                  {#each section.selectFields as field (field.key)}
                    <div class="space-y-2 rounded-xl border border-sidebar-border/60 bg-sidebar-background/70 p-3">
                      <div class="flex items-center gap-1">
                        <Label class="block text-xs font-medium uppercase tracking-wide text-sidebar-foreground/70">
                          {field.label}
                        </Label>

                        <Tooltip.Root>
                          <Tooltip.Trigger >
                            <button
                              type="button"
                              class="inline-flex items-center text-muted-foreground transition-colors hover:text-foreground"
                              aria-label={`Help for ${field.label}`}
                            >
                              <HugeiconsIcon icon={HelpCircleIcon} strokeWidth={2} class="size-3.5" />
                            </button>
                          </Tooltip.Trigger>

                          <Tooltip.Content side="top" align="center" class="max-w-xs">
                            <p class="text-xs">{field.description}</p>
                          </Tooltip.Content>
                        </Tooltip.Root>
                      </div>

                      <Select.Root
                        type="single"
                        value={getSelectValue(field)}
                        onValueChange={(value: string) => {
                          setText(field.key, value);
                        }}
                      >
                        <Select.Trigger class="w-full rounded-md border border-sidebar-border/60 bg-background px-3 py-2 text-left text-sm outline-none ring-0">
                          {getSelectValue(field)}
                        </Select.Trigger>

                        <Select.Content>
                          <Select.Group>
                            {#each field.options as option (option)}
                              <Select.Item value={option}>{option}</Select.Item>
                            {/each}
                          </Select.Group>
                        </Select.Content>
                      </Select.Root>
                    </div>
                  {/each}
                </div>
              {/if}

              {#if section.sliderFields && section.sliderFields.length}
                <div class="grid gap-3 md:grid-cols-2">
                  {#each section.sliderFields as field (field.key)}
                    <div class="space-y-2 rounded-xl border border-sidebar-border/60 bg-sidebar-background/70 p-3">
                      <div class="flex items-center justify-between gap-3">
                        <div class="flex items-center gap-1">
                          <Label class="block text-xs font-medium uppercase tracking-wide text-sidebar-foreground/70">
                            {field.label}
                          </Label>

                          <Tooltip.Root>
                            <Tooltip.Trigger >
                              <button
                                type="button"
                                class="inline-flex items-center text-muted-foreground transition-colors hover:text-foreground"
                                aria-label={`Help for ${field.label}`}
                              >
                                <HugeiconsIcon icon={HelpCircleIcon} strokeWidth={2} class="size-3.5" />
                              </button>
                            </Tooltip.Trigger>

                            <Tooltip.Content side="top" align="center" class="max-w-xs">
                              <p class="text-xs">{field.description}</p>
                            </Tooltip.Content>
                          </Tooltip.Root>
                        </div>

                        <span class="text-xs text-muted-foreground">
                          {field.format ? field.format(getRangeValue(field)) : getRangeValue(field)}
                          {#if field.unit}
                            {field.unit}
                          {/if}
                        </span>
                      </div>

                      <input
                        type="range"
                        min={field.min}
                        max={field.max}
                        step={field.step ?? 1}
                        value={getRangeValue(field)}
                        oninput={(event) => {
                          setRangeValue(field, Number((event.currentTarget as HTMLInputElement).value));
                        }}
                        class="w-full accent-primary"
                      />
                    </div>
                  {/each}
                </div>
              {/if}

              {#if section.booleanFields && section.booleanFields.length}
                <div class="grid gap-3 md:grid-cols-2">
                  {#each section.booleanFields as field (field.key)}
                    <div class="rounded-xl border border-sidebar-border/60 bg-sidebar-background/70 p-3">
                      <div class="flex items-center justify-between gap-3">
                        <div class="flex items-start gap-1">
                          <div>
                            <Label class="block text-xs font-medium uppercase tracking-wide text-sidebar-foreground/70">
                              {field.label}
                            </Label>
                            <p class="text-xs text-muted-foreground">{field.description}</p>
                          </div>

                          <Tooltip.Root>
                            <Tooltip.Trigger>
                              <button
                                type="button"
                                class="mt-0.5 inline-flex items-center text-muted-foreground transition-colors hover:text-foreground"
                                aria-label={`Help for ${field.label}`}
                              >
                                <HugeiconsIcon icon={HelpCircleIcon} strokeWidth={2} class="size-3.5" />
                              </button>
                            </Tooltip.Trigger>

                            <Tooltip.Content side="top" align="center" class="max-w-xs">
                              <p class="text-xs">{field.description}</p>
                            </Tooltip.Content>
                          </Tooltip.Root>
                        </div>

                        <label class="relative inline-flex cursor-pointer items-center">
                          <input
                            type="checkbox"
                            checked={getBoolean(field.key)}
                            onchange={(event) => {
                              setBoolean(field.key, (event.currentTarget as HTMLInputElement).checked);
                            }}
                            class="peer sr-only"
                          />
                          <div class="peer h-6 w-11 rounded-full bg-muted transition-colors peer-checked:bg-primary"></div>
                          <div class="absolute left-0.5 top-0.5 h-5 w-5 rounded-full bg-background shadow transition-transform peer-checked:translate-x-5"></div>
                        </label>
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}
            </section>
          {/each}
        </div>

        <div class="flex items-center justify-end gap-2 border-t border-sidebar-border/60 p-4">
          <button
            type="button"
            onclick={close}
            class="rounded-lg border border-sidebar-border/60 px-4 py-2 text-sm transition-colors hover:bg-muted"
          >
            Cancel
          </button>
          <button
            type="button"
            onclick={save}
            class="rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:opacity-90"
          >
            Save
          </button>
        </div>
      </div>
    </div>
  </Tooltip.Provider>
{/if}
