"use client";

import React, { useRef, useEffect, useState } from "react";
import * as d3 from "d3";

interface OrderLevel {
    price: number;
    size: number;
}

interface D3DepthChartProps {
    bids: OrderLevel[];
    asks: OrderLevel[];
}

export default function D3DepthChart({ bids, asks }: D3DepthChartProps) {
    const containerRef = useRef<HTMLDivElement>(null);
    const svgRef = useRef<SVGSVGElement>(null);
    const [dimensions, setDimensions] = useState({ width: 0, height: 0 });

    useEffect(() => {
        const observeTarget = containerRef.current;
        if (!observeTarget) return;

        const resizeObserver = new ResizeObserver((entries) => {
            for (const entry of entries) {
                setDimensions({
                    width: entry.contentRect.width,
                    height: entry.contentRect.height,
                });
            }
        });

        resizeObserver.observe(observeTarget);
        return () => resizeObserver.disconnect();
    }, []);

    useEffect(() => {
        if (!svgRef.current || dimensions.width === 0 || dimensions.height === 0) return;

        const svg = d3.select(svgRef.current);
        svg.selectAll("*").remove();

        const margin = { top: 20, right: 10, bottom: 30, left: 10 };
        const width = dimensions.width - margin.left - margin.right;
        const height = dimensions.height - margin.top - margin.bottom;

        const g = svg
            .append("g")
            .attr("transform", `translate(${margin.left},${margin.top})`);

        // Process data for cumulative depth
        // Bids: price descending, cumulative size increases as price decreases
        let cumulativeBid = 0;
        const bidPoints = bids.map(d => {
            cumulativeBid += Number(d.size);
            return { price: Number(d.price), total: cumulativeBid };
        });

        // Asks: price ascending, cumulative size increases as price increases
        let cumulativeAsk = 0;
        const askPoints = asks.map(d => {
            cumulativeAsk += Number(d.size);
            return { price: Number(d.price), total: cumulativeAsk };
        });

        if (bidPoints.length === 0 && askPoints.length === 0) return;

        const allPrices = [...bidPoints, ...askPoints].map(d => d.price);
        const x = d3.scaleLinear()
            .domain([d3.min(allPrices)! * 0.999, d3.max(allPrices)! * 1.001])
            .range([0, width]);

        const maxTotal = Math.max(
            d3.max(bidPoints, d => d.total) || 0,
            d3.max(askPoints, d => d.total) || 0
        );

        const y = d3.scaleLinear()
            .domain([0, maxTotal * 1.1])
            .range([height, 0]);

        // Areas
        const areaBuilder = d3.area<{ price: number; total: number }>()
            .curve(d3.curveStepBefore)
            .x(d => x(d.price))
            .y0(height)
            .y1(d => y(d.total));

        // Draw Bids (Green)
        if (bidPoints.length > 0) {
            g.append("path")
                .datum(bidPoints)
                .attr("fill", "#10b98133")
                .attr("stroke", "#10b981")
                .attr("stroke-width", 2)
                .attr("d", areaBuilder);
        }

        // Draw Asks (Red)
        if (askPoints.length > 0) {
            const askArea = d3.area<{ price: number; total: number }>()
                .curve(d3.curveStepAfter)
                .x(d => x(d.price))
                .y0(height)
                .y1(d => y(d.total));

            g.append("path")
                .datum(askPoints)
                .attr("fill", "#f43f5e33")
                .attr("stroke", "#f43f5e")
                .attr("stroke-width", 2)
                .attr("d", askArea);
        }

        // Axes
        g.append("g")
            .attr("transform", `translate(0,${height})`)
            .call(d3.axisBottom(x).ticks(5).tickFormat(d => d.valueOf().toString()))
            .attr("color", "#ffffff44")
            .attr("font-size", "10px")
            .select(".domain").remove();

    }, [bids, asks, dimensions]);

    return (
        <div ref={containerRef} className="w-full h-full">
            <svg ref={svgRef} width={dimensions.width} height={dimensions.height} />
        </div>
    );
}
