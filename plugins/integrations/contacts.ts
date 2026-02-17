import { invoke } from "@tauri-apps/api/core";
import type { GeniePlugin, SearchResult } from "../../src/types";

interface Contact {
  name: string;
  email?: string;
  phone?: string;
}

const CACHE_TTL = 5 * 60 * 1000;
const PREFIXES = ["contact ", "contacts ", "contact:", "contacts:"];
let contactsCache: Contact[] = [];
let cacheTimestamp = 0;
let pendingLoad: Promise<Contact[]> | null = null;

function parseContactsQuery(raw: string): { query: string; forced: boolean } {
  const trimmed = raw.trim();
  const lower = trimmed.toLowerCase();
  
  // Check for exact "contact" or "contacts" command
  if (lower === "contact" || lower === "contacts") return { query: "", forced: true };
  
  // Check for prefix patterns
  for (const prefix of PREFIXES) {
    if (lower.startsWith(prefix)) {
      return { query: trimmed.slice(prefix.length).trim(), forced: true };
    }
  }
  
  // For non-prefixed queries, search if query is at least 2 chars
  return { query: trimmed, forced: false };
}

function matchesContact(contact: Contact, query: string): boolean {
  if (!query) return true;
  const q = query.toLowerCase();
  return (
    contact.name.toLowerCase().includes(q) ||
    (contact.email ?? "").toLowerCase().includes(q) ||
    (contact.phone ?? "").toLowerCase().includes(q)
  );
}

function scoreContact(contact: Contact, query: string, index: number, forced: boolean): number {
  // Higher base score when explicitly searching contacts
  const baseScore = forced ? 800 : 580;
  
  if (!query) return baseScore - index;
  const q = query.toLowerCase();
  if (contact.name.toLowerCase().startsWith(q)) return baseScore + 160 - index;
  return baseScore + 40 - index;
}

async function loadContacts(): Promise<Contact[]> {
  const now = Date.now();
  if (contactsCache.length > 0 && now - cacheTimestamp < CACHE_TTL) return contactsCache;
  if (pendingLoad) return pendingLoad;
  
  const timeout = new Promise<Contact[]>((resolve) =>
    setTimeout(() => resolve(contactsCache), 3000)
  );
  
  const load = invoke<Contact[]>("get_contacts")
    .then((contacts) => {
      console.log(`[Contacts] Loaded ${contacts.length} contacts`);
      return contacts.filter((c) => c.name?.trim().length > 0);
    })
    .catch((err) => {
      console.error("[Contacts] Failed to load:", err);
      return contactsCache;
    });
  
  pendingLoad = Promise.race([load, timeout]).finally(() => {
    pendingLoad = null;
  });
  
  contactsCache = await pendingLoad;
  cacheTimestamp = Date.now();
  return contactsCache;
}

export const contactsPlugin: GeniePlugin = {
  id: "integration:contacts",
  name: "Contacts",
  icon: "users",
  keyword: "contact",
  debounceMs: 150,

  async onSearch(rawQuery: string): Promise<SearchResult[]> {
    const parsed = parseContactsQuery(rawQuery);
    
    // For non-forced queries, require at least 2 characters
    if (!parsed.forced && parsed.query.length < 2) return [];
    
    const contacts = await loadContacts();
    
    if (contacts.length === 0) {
      console.log("[Contacts] No contacts available");
      return [];
    }
    
    const matches = contacts.filter((contact) => matchesContact(contact, parsed.query));
    
    console.log(`[Contacts] Query: "${rawQuery}", Matches: ${matches.length}`);
    
    if (matches.length === 0) return [];

    return matches.slice(0, 8).map((contact, index) => ({
      id: `contact:${contact.name}:${contact.email ?? contact.phone ?? index}`,
      title: contact.name,
      subtitle: contact.email || contact.phone || "Contact",
      category: "CONTACT",
      icon: "users",
      action_data: contact.email || contact.phone || "",
      score: scoreContact(contact, parsed.query, index, parsed.forced),
    }));
  },

  async onAction(result: SearchResult): Promise<void> {
    if (result.action_data.includes("@")) {
      window.location.href = `mailto:${result.action_data}`;
      return;
    }
    if (result.action_data) {
      await navigator.clipboard.writeText(result.action_data);
    }
  },
};
