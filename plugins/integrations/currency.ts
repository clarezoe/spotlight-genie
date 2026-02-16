import type { GeniePlugin, SearchResult } from "../../src/types";

const COMMON_CURRENCIES = ["USD", "EUR", "GBP", "JPY", "CNY", "KRW", "CAD", "AUD", "CHF", "SEK"];

let ratesCache: Record<string, number> | null = null;
let cacheTimestamp = 0;
const CACHE_TTL = 3600000;

async function fetchRates(base: string): Promise<Record<string, number>> {
  const now = Date.now();
  if (ratesCache && now - cacheTimestamp < CACHE_TTL) {
    return ratesCache;
  }
  try {
    const res = await fetch(
      `https://api.exchangerate-api.com/v4/latest/${base}`
    );
    const data = await res.json();
    ratesCache = data.rates;
    cacheTimestamp = now;
    return data.rates;
  } catch {
    return ratesCache ?? {};
  }
}

function parseQuery(query: string): { amount: number; from: string; to: string } | null {
  const match = query.match(/^(\d+(?:\.\d+)?)\s*([A-Za-z]{3})\s+(?:to\s+)?([A-Za-z]{3})$/i);
  if (!match) return null;
  return {
    amount: parseFloat(match[1]),
    from: match[2].toUpperCase(),
    to: match[3].toUpperCase(),
  };
}

export const currencyPlugin: GeniePlugin = {
  id: "integration:currency",
  name: "Currency",
  icon: "dollar-sign",
  debounceMs: 200,

  async onSearch(query: string): Promise<SearchResult[]> {
    let q = query.trim();
    if (!q) return [];
    if (q.toLowerCase().startsWith("cc ")) {
      q = q.slice(3).trim();
    }
    const parsed = parseQuery(q);
    if (!parsed) return [];

    const rates = await fetchRates(parsed.from);
    const rate = rates[parsed.to];
    if (!rate) {
      return [
        {
          id: "currency:error",
          title: `Unknown currency: ${parsed.to}`,
          subtitle: `Supported: ${COMMON_CURRENCIES.join(", ")}`,
          category: "CURRENCY",
          icon: "dollar-sign",
          action_data: "",
          score: 300,
        },
      ];
    }

    const converted = (parsed.amount * rate).toFixed(2);
    return [
      {
        id: "currency:result",
        title: `${parsed.amount} ${parsed.from} = ${converted} ${parsed.to}`,
        subtitle: `Rate: 1 ${parsed.from} = ${rate.toFixed(4)} ${parsed.to}`,
        category: "CURRENCY",
        icon: "dollar-sign",
        action_data: converted,
        score: 900,
      },
    ];
  },

  async onAction(result: SearchResult): Promise<void> {
    if (result.action_data) {
      await navigator.clipboard.writeText(result.action_data);
    }
  },
};
