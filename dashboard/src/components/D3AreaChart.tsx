"use client";

import React, { useRef, useEffect, useState } from "react";
import * as d3 from "d3";

interface DataPoint {
    time: string;
    [key: string]: number | string;
}

interface D3AreaChartProps {
    data: DataPoint[];
    dataKey: string;
    color: string;
    gradientId: string;
    name: string;
    pivotPrice?: number;
    buyChannelWidth?: number;
    sellChannelWidth?: number;
}

export default function D3AreaChart({
    data,
    dataKey,
    color,
    gradientId,
    name,
    pivotPrice,
    buyChannelWidth,
    sellChannelWidth
}: D3AreaChartProps) {
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

        // Tooltip cleanup: ensure no stale tooltips are left from previous renders
        d3.select(containerRef.current).selectAll(".d3-tooltip").remove();

        const svg = d3.select(svgRef.current);
        svg.selectAll("*").remove();

        // ... (rest of setup)
        const margin = { top: 40, right: 30, bottom: 40, left: 60 };
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
            .domain([
                d3.min(data, (d) => Number(d[dataKey]))! * 0.98,
                d3.max(data, (d) => Number(d[dataKey]))! * 1.02
            ])
            .range([height, 0]);

        // Find fallback pivot: Last price that is not zero
        const lastValidPrice = [...data].reverse().find(d => Number(d[dataKey]) > 0);
        const effectivePivot = (pivotPrice && pivotPrice > 0)
            ? pivotPrice
            : (lastValidPrice ? Number(lastValidPrice[dataKey]) : 0);

        if (effectivePivot && effectivePivot > 0 && buyChannelWidth && sellChannelWidth) {
            const lower = effectivePivot * (1 - buyChannelWidth / 100);
            const upper = effectivePivot * (1 + sellChannelWidth / 100);

            // Ensure channel is within y scale with a buffer
            const currentDomain = y.domain();
            const yMin = Math.min(currentDomain[0], lower * 0.95);
            const yMax = Math.max(currentDomain[1], upper * 1.05);
            y.domain([yMin, yMax]);

            const yPivot = y(effectivePivot);
            const yUpper = y(upper);
            const yLower = y(lower);

            // Draw Sell Zone (Red) - Above Pivot
            g.append("rect")
                .attr("x", 0)
                .attr("y", yUpper)
                .attr("width", width)
                .attr("height", Math.max(0, yPivot - yUpper))
                .attr("fill", "#f43f5e") // Rose/Red 500
                .attr("fill-opacity", 0.15)
                .attr("stroke", "#f43f5e")
                .attr("stroke-opacity", 0.4)
                .attr("stroke-dasharray", "3 3");

            // Draw Buy Zone (Green) - Below Pivot
            g.append("rect")
                .attr("x", 0)
                .attr("y", yPivot)
                .attr("width", width)
                .attr("height", Math.max(0, yLower - yPivot))
                .attr("fill", "#10b981") // Emerald/Green 500
                .attr("fill-opacity", 0.15)
                .attr("stroke", "#10b981")
                .attr("stroke-opacity", 0.4)
                .attr("stroke-dasharray", "3 3");

            // Draw Pivot Line (Dashed Orange)
            g.append("line")
                .attr("x1", 0)
                .attr("y1", yPivot)
                .attr("x2", width)
                .attr("y2", yPivot)
                .attr("stroke", "#f59e0b") // Amber/Orange 500
                .attr("stroke-width", 1.5)
                .attr("stroke-dasharray", "5 5")
                .attr("stroke-opacity", 0.6);
        }

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

        const yAxisFormatter = (v: number) => {
            if (v === 0) return "0";
            if (v < 1) return v.toFixed(9);
            return v.toFixed(6);
        };

        g.append("g")
            .call(d3.axisLeft(y).ticks(5).tickFormat((d) => yAxisFormatter(d as number)))
            .attr("color", "#ffffff66")
            .attr("font-size", "10px")
            .select(".domain").remove();

        // Gradient
        const gradient = svg.append("defs")
            .append("linearGradient")
            .attr("id", gradientId)
            .attr("x1", "0%")
            .attr("y1", "0%")
            .attr("x2", "0%")
            .attr("y2", "100%");

        gradient.append("stop")
            .attr("offset", "5%")
            .attr("stop-color", color)
            .attr("stop-opacity", 0.4);

        gradient.append("stop")
            .attr("offset", "95%")
            .attr("stop-color", color)
            .attr("stop-opacity", 0);

        // Area
        const area = d3.area<DataPoint>()
            .x(d => x(d.time)!)
            .y0(height)
            .y1(d => y(Number(d[dataKey])))
            .curve(d3.curveMonotoneX);

        g.append("path")
            .datum(data)
            .attr("fill", `url(#${gradientId})`)
            .attr("d", area);

        // Line
        const line = d3.line<DataPoint>()
            .x(d => x(d.time)!)
            .y(d => y(Number(d[dataKey])))
            .curve(d3.curveMonotoneX);

        g.append("path")
            .datum(data)
            .attr("fill", "none")
            .attr("stroke", color)
            .attr("stroke-width", 1.5)
            .attr("d", line);

        // Tooltip logic
        const tooltip = d3.select(containerRef.current)
            .append("div")
            .attr("class", "d3-tooltip")
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

        const focus = g.append("g")
            .style("display", "none");

        focus.append("circle")
            .attr("r", 5)
            .attr("fill", color)
            .attr("stroke", "#fff")
            .attr("stroke-width", 2);

        svg.on("mousemove", (event) => {
            const [mx, my] = d3.pointer(event);
            const mouseX = mx - margin.left;

            // Find nearest point
            const domain = x.domain();
            const range = x.range();
            const proportions = domain.map((_, i) => (range[i] || 0));

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
                focus.style("display", null)
                    .attr("transform", `translate(${x(d.time)},${y(Number(d[dataKey]))})`);

                const [cx, cy] = d3.pointer(event, containerRef.current);
                tooltip.style("visibility", "visible")
                    .html(`
                    <div style="text-transform: uppercase; letter-spacing: 0.1em; color: #94a3b8; margin-bottom: 4px;">${d.time}</div>
                    <div style="font-weight: bold; color: ${color};">${name}: ${Number(d[dataKey]).toLocaleString(undefined, { minimumFractionDigits: 6, maximumFractionDigits: 9 })}</div>
                `)
                    .style("top", `${cy - 120}px`)
                    .style("left", `${cx + 20}px`);
            }
        });

        svg.on("mouseleave", () => {
            focus.style("display", "none");
            tooltip.style("visibility", "hidden");
            // Final safety cleanup: remove any tooltips that might have been stranded
            d3.select(containerRef.current).selectAll(".d3-tooltip").style("visibility", "hidden");
        });

    }, [data, dimensions, color, dataKey, gradientId, name, pivotPrice, buyChannelWidth, sellChannelWidth]);

    return (
        <div ref={containerRef} className="w-full h-full relative">
            <svg ref={svgRef} width={dimensions.width} height={dimensions.height} />
        </div>
    );
}
