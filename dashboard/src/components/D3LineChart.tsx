"use client";

import React, { useRef, useEffect, useState } from "react";
import * as d3 from "d3";

interface D3LineChartProps {
    data: any[];
    services: string[];
}

export default function D3LineChart({ data, services }: D3LineChartProps) {
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
        if (!svgRef.current || dimensions.width === 0 || dimensions.height === 0 || data.length === 0) return;

        const svg = d3.select(svgRef.current);
        svg.selectAll("*").remove();

        const margin = { top: 40, right: 120, bottom: 40, left: 60 };
        const width = dimensions.width - margin.left - margin.right;
        const height = dimensions.height - margin.top - margin.bottom;

        const g = svg
            .append("g")
            .attr("transform", `translate(${margin.left},${margin.top})`);

        // Scales
        const x = d3.scalePoint()
            .domain(data.map(d => d.time))
            .range([0, width]);

        const y = d3.scaleLinear()
            .domain([0, d3.max(data, d => {
                return Math.max(...services.map(s => d[s] || 0));
            }) * 1.1])
            .range([height, 0]);

        const color = d3.scaleOrdinal<string>()
            .domain(services)
            .range(['#22d3ee', '#a855f7', '#fbbf24', '#f87171', '#34d399']);

        // Grid lines
        g.append("g")
            .attr("class", "grid")
            .attr("stroke", "#ffffff11")
            .attr("stroke-dasharray", "3 3")
            .call(d3.axisLeft(y)
                .tickSize(-width)
                .tickFormat(() => "")
            )
            .select(".domain").remove();

        // Axes
        g.append("g")
            .attr("transform", `translate(0,${height})`)
            .call(d3.axisBottom(x).tickValues(x.domain().filter((d, i) => !(i % Math.ceil(data.length / 6)))))
            .attr("color", "#ffffff66")
            .attr("font-size", "12px")
            .select(".domain").remove();

        g.append("g")
            .call(d3.axisLeft(y).ticks(5))
            .attr("color", "#ffffff66")
            .attr("font-size", "12px")
            .select(".domain").remove();

        // Lines
        services.forEach(service => {
            const line = d3.line<any>()
                .x(d => x(d.time)!)
                .y(d => y(d[service] || 0))
                .curve(d3.curveMonotoneX)
                .defined(d => d[service] !== undefined);

            g.append("path")
                .datum(data)
                .attr("fill", "none")
                .attr("stroke", color(service))
                .attr("stroke-width", 2)
                .attr("d", line);
        });

        // Legend
        const legend = g.append("g")
            .attr("transform", `translate(${width + 20}, 0)`);

        services.forEach((service, i) => {
            const legendRow = legend.append("g")
                .attr("transform", `translate(0, ${i * 20})`);

            legendRow.append("circle")
                .attr("r", 5)
                .attr("fill", color(service));

            legendRow.append("text")
                .attr("x", 12)
                .attr("y", 5)
                .attr("fill", "#fff")
                .attr("font-size", "10px")
                .attr("font-weight", "bold")
                .style("text-transform", "uppercase")
                .text(service);
        });

        // Tooltip logic
        const tooltip = d3.select(containerRef.current)
            .append("div")
            .style("position", "absolute")
            .style("visibility", "hidden")
            .style("background", "#0f172a")
            .style("border", "1px solid #ffffff22")
            .style("border-radius", "16px")
            .style("padding", "12px")
            .style("color", "#fff")
            .style("font-size", "12px")
            .style("pointer-events", "none")
            .style("backdrop-filter", "blur(10px)")
            .style("z-index", "100");

        const mouseG = g.append("g")
            .style("display", "none");

        mouseG.append("line")
            .attr("stroke", "#ffffff33")
            .attr("stroke-width", 1)
            .attr("y1", 0)
            .attr("y2", height);

        svg.on("mousemove", (event) => {
            const [mx, my] = d3.pointer(event);
            const mouseX = mx - margin.left;

            const domain = x.domain();
            const range = x.range();
            const proportions = domain.map((_, i) => (range[i] ?? 0));

            let nearestIndex = 0;
            let minDiff = Infinity;

            proportions.forEach((pos, i) => {
                const diff = Math.abs(pos - mouseX);
                if (diff < minDiff) {
                    minDiff = diff;
                    nearestIndex = i;
                }
            });

            const d = data[nearestIndex];
            if (d && mouseX >= 0 && mouseX <= width) {
                mouseG.style("display", null)
                    .attr("transform", `translate(${x(d.time)},0)`);

                let tooltipHtml = `<div style="text-transform: uppercase; letter-spacing: 0.1em; color: #94a3b8; margin-bottom: 4px;">${d.time}</div>`;
                services.forEach(s => {
                    if (d[s]) {
                        tooltipHtml += `<div style="font-weight: bold; color: ${color(s)};">${s}: ${d[s]} ms</div>`;
                    }
                });

                tooltip.style("visibility", "visible")
                    .html(tooltipHtml)
                    .style("top", `${event.pageY - 120}px`)
                    .style("left", `${event.pageX + 10}px`);
            }
        });

        svg.on("mouseleave", () => {
            mouseG.style("display", "none");
            tooltip.style("visibility", "hidden");
        });

    }, [data, dimensions, services]);

    return (
        <div ref={containerRef} className="w-full h-full relative">
            <svg ref={svgRef} width={dimensions.width} height={dimensions.height} />
        </div>
    );
}
