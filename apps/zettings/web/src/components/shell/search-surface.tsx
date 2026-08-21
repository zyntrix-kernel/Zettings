/**
 * Search surface (PLAN §5): instant, registry-backed, keyboard-operable.
 *
 * Results come from the Rust `search_registry` command (weighted kernel).
 * Stale responses are discarded via a sequence guard; the result count is a
 * concise polite status. Arrow keys move through results; Enter opens;
 * Escape clears and returns focus to the input.
 */
import { useEffect, useRef, useState } from "react";
import type { SearchHitDto, SearchResponseDto } from "@zettings/bindings";
import { invokeIpc } from "../../lib/ipc";

export interface SearchSurfaceProps {
  /** Opens the chosen hit's deep link in the shell router. */
  onOpen: (hit: SearchHitDto) => void;
}

export function SearchSurface({ onOpen }: SearchSurfaceProps) {
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHitDto[]>([]);
  const [statusText, setStatusText] = useState("");
  const [activeIndex, setActiveIndex] = useState(-1);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);
  const sequence = useRef(0);

  useEffect(() => {
    // Global Ctrl+F focuses search (keyboard contract, DESIGN.md §8.4).
    const onKey = (event: KeyboardEvent): void => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "f") {
        event.preventDefault();
        inputRef.current?.focus();
        inputRef.current?.select();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  useEffect(() => {
    const trimmed = query.trim();
    if (trimmed === "") {
      sequence.current += 1;
      setHits([]);
      setStatusText("");
      setActiveIndex(-1);
      return;
    }
    const ticket = ++sequence.current;
    const timer = window.setTimeout(() => {
      invokeIpc<SearchResponseDto>("search_registry", { query: trimmed })
        .then((response) => {
          if (ticket !== sequence.current) return; // stale
          setHits(response.hits);
          setActiveIndex(-1);
          setStatusText(
            response.hits.length === 0
              ? `No settings match “${response.query}”.`
              : `${response.hits.length} setting${response.hits.length === 1 ? "" : "s"} found.`,
          );
        })
        .catch((cause: unknown) => {
          if (ticket !== sequence.current) return;
          setHits([]);
          const message = cause instanceof Error ? cause.message : String(cause);
          setStatusText(`Search failed: ${message}`);
        });
    }, 120);
    return () => window.clearTimeout(timer);
  }, [query]);

  const openHit = (hit: SearchHitDto): void => {
    onOpen(hit);
    setQuery("");
    setStatusText("");
    inputRef.current?.blur();
  };

  const onKeyDown = (event: React.KeyboardEvent): void => {
    if (event.key === "Escape") {
      if (query !== "") {
        setQuery("");
        setStatusText("");
      } else {
        inputRef.current?.blur();
      }
      return;
    }
    if (hits.length === 0) return;
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setActiveIndex((i) => Math.min(hits.length - 1, i + 1));
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      setActiveIndex((i) => Math.max(0, i - 1));
    } else if (event.key === "Enter" && activeIndex >= 0) {
      event.preventDefault();
      const hit = hits[activeIndex];
      if (hit !== undefined) openHit(hit);
    }
  };

  return (
    <div className="zdl-search">
      <input
        ref={inputRef}
        type="search"
        role="combobox"
        aria-expanded={hits.length > 0}
        aria-controls="search-results"
        aria-describedby="search-status"
        aria-autocomplete="list"
        aria-label="Search settings"
        placeholder="Find a setting"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        onKeyDown={onKeyDown}
      />
      <p id="search-status" role="status" className="visually-hidden">
        {statusText}
      </p>
      <ul id="search-results" ref={listRef} className="zdl-search__list" role="listbox" aria-label="Search results" hidden={hits.length === 0}>
        {hits.map((hit, index) => (
          <li key={hit.setting_id} role="option" aria-selected={index === activeIndex}>
            <button
              type="button"
              className="zdl-search__result"
              data-active={index === activeIndex || undefined}
              onMouseEnter={() => setActiveIndex(index)}
              onClick={() => openHit(hit)}
            >
              <span className="zdl-card__title">{hit.title}</span>
              <span className="zdl-card__description">{hit.description}</span>
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
