"use client";

import { useRef, useState, useEffect } from "react";
import { Media } from "@/types";
import ContentCard from "./ContentCard";
import {
  ChevronLeftIcon,
  ChevronRightIcon,
} from "@heroicons/react/24/outline";

interface ContentRowProps {
  title: string;
  items: Media[];
  size?: "sm" | "md" | "lg";
  href?: string;
}

export default function ContentRow({
  title,
  items,
  size = "md",
  href,
}: ContentRowProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [canScrollLeft, setCanScrollLeft] = useState(false);
  const [canScrollRight, setCanScrollRight] = useState(true);

  const updateScrollButtons = () => {
    if (!scrollRef.current) return;
    const { scrollLeft, scrollWidth, clientWidth } = scrollRef.current;
    setCanScrollLeft(scrollLeft > 10);
    setCanScrollRight(scrollLeft < scrollWidth - clientWidth - 10);
  };

  useEffect(() => {
    updateScrollButtons();
    const el = scrollRef.current;
    if (el) {
      el.addEventListener("scroll", updateScrollButtons, { passive: true });
      return () => el.removeEventListener("scroll", updateScrollButtons);
    }
  }, [items]);

  const scroll = (direction: "left" | "right") => {
    if (!scrollRef.current) return;
    const width = scrollRef.current.clientWidth;
    scrollRef.current.scrollBy({
      left: direction === "right" ? width * 0.75 : -width * 0.75,
      behavior: "smooth",
    });
  };

  if (!items.length) return null;

  return (
    <section className="relative py-4 lg:py-6">
      <div className="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between mb-3 lg:mb-4">
          <h2 className="text-lg lg:text-xl font-bold text-white">{title}</h2>
          {href && (
            <a
              href={href}
              className="text-sm text-accent-light hover:text-accent transition-colors"
            >
              View All
            </a>
          )}
        </div>
      </div>

      <div className="relative group/row">
        {canScrollLeft && (
          <button
            onClick={() => scroll("left")}
            className="absolute left-0 top-0 bottom-0 w-12 lg:w-16 z-10
                       bg-gradient-to-r from-surface/95 to-transparent
                       flex items-center justify-start pl-2
                       opacity-0 group-hover/row:opacity-100 transition-opacity duration-300"
          >
            <div className="w-9 h-9 rounded-full bg-surface-200/80 backdrop-blur-sm flex items-center justify-center border border-white/10 hover:bg-surface-300/80 transition-colors">
              <ChevronLeftIcon className="w-5 h-5 text-white" />
            </div>
          </button>
        )}

        {canScrollRight && (
          <button
            onClick={() => scroll("right")}
            className="absolute right-0 top-0 bottom-0 w-12 lg:w-16 z-10
                       bg-gradient-to-l from-surface/95 to-transparent
                       flex items-center justify-end pr-2
                       opacity-0 group-hover/row:opacity-100 transition-opacity duration-300"
          >
            <div className="w-9 h-9 rounded-full bg-surface-200/80 backdrop-blur-sm flex items-center justify-center border border-white/10 hover:bg-surface-300/80 transition-colors">
              <ChevronRightIcon className="w-5 h-5 text-white" />
            </div>
          </button>
        )}

        <div
          ref={scrollRef}
          className="flex gap-2.5 lg:gap-3 overflow-x-auto no-scrollbar px-4 sm:px-6 lg:px-8 scroll-smooth"
        >
          {items.map((item) => (
            <ContentCard key={`${item.type}-${item.id}`} media={item} size={size} />
          ))}
        </div>
      </div>
    </section>
  );
}
