import type { Aria2Options } from "$lib/aria2/types";

export type DownloadKind = "torrent" | "uri" | "all";

export interface ItemFieldDefinition {
  key: keyof Aria2Options;
  label: string;
  type: "string" | "number" | "boolean" | "select";
  description: string;
  options?: { label: string; value: string }[]; // For select inputs
}

export interface ItemSettingsSection {
  id: string;
  title: string;
  description: string;
  icon: string; // Map to your SettingIcons keys
  visibleFor: DownloadKind[]; // Determines if this section shows up for Torrents or URIs
  fields: ItemFieldDefinition[];
}

export const itemSettingsSections: ItemSettingsSection[] = [
  {
    id: "general",
    title: "General Settings",
    description: "Basic download path and limits.",
    icon: "server", 
    visibleFor: ["all", "torrent", "uri"],
    fields: [
      {
        key: "dir",
        label: "Directory",
        type: "string",
        description: "The directory to store the downloaded file.",
      },
      {
        key: "out",
        label: "Output File Name",
        type: "string",
        description: "The file name of the downloaded file. (May not work for multi-file torrents).",
      },
      {
        key: "max-download-limit",
        label: "Max Download Limit",
        type: "string",
        description: "Set max download speed (e.g., 500K, 2M). 0 means unrestricted.",
      },
      {
        key: "max-upload-limit",
        label: "Max Upload Limit",
        type: "string",
        description: "Set max upload speed (e.g., 500K, 2M). 0 means unrestricted.",
      }
    ]
  },
  {
    id: "http-ftp",
    title: "HTTP / FTP / URI",
    description: "Settings specific to direct URL downloads.",
    icon: "globe",
    visibleFor: ["uri", "all"],
    fields: [
      {
        key: "max-connection-per-server",
        label: "Max Connections Per Server",
        type: "number",
        description: "The maximum number of connections to one server for this download.",
      },
      {
        key: "split",
        label: "Split",
        type: "number",
        description: "Download a file using N connections.",
      },
      {
        key: "user-agent",
        label: "User Agent",
        type: "string",
        description: "Set user agent for HTTP(S) downloads.",
      },
      {
        key: "referer",
        label: "Referer",
        type: "string",
        description: "Set an HTTP referer.",
      }
    ]
  },
  {
    id: "bittorrent",
    title: "BitTorrent",
    description: "Settings specific to active torrent downloads.",
    icon: "activity",
    visibleFor: ["torrent", "all"],
    fields: [
      {
        key: "bt-max-peers",
        label: "Max Peers",
        type: "number",
        description: "Specify the maximum number of peers for this torrent.",
      },
      {
        key: "bt-request-peer-speed-limit",
        label: "Request Peer Speed Limit",
        type: "string",
        description: "If the whole download speed exceeds this, aria2 won't send requests to peers.",
      },
      {
        key: "seed-ratio",
        label: "Seed Ratio",
        type: "string",
        description: "Specify share ratio. Seed completed torrents until ratio is reached.",
      },
      {
        key: "seed-time",
        label: "Seed Time",
        type: "string",
        description: "Specify seeding time in minutes.",
      }
    ]
  }
];
