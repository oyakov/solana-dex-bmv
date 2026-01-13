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
    channelWidth?: number;
}

export default function D3AreaChart({
    data,
    dataKey,
    color,
    gradientId,
    name,
    pivotPrice,
    channelWidth
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

        const svg = d3.select(svgRef.current);
        svg.selectAll("*").remove();

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
                d3.min(data, (d) => Number(d[dataKey]))! * 0.999,
                d3.max(data, (d) => Number(d[dataKey]))! * 1.001
            ])
            .range([height, 0]);

        if (pivotPrice && channelWidth) {
            const lower = pivotPrice * (1 - channelWidth / 100);
            const upper = pivotPrice * (1 + channelWidth / 100);

            // Ensure channel is within y scale
            const yMin = Math.min(y.domain()[0], lower);
            const yMax = Math.max(y.domain()[1], upper);
            y.domain([yMin, yMax]);

            // Draw Reference Area (Channel)
            g.append("rect")
                .attr("x", 0)
                .attr("y", y(upper))
                .attr("width", width)
                .attr("height", y(lower) - y(upper))
                .attr("fill", color)
                .attr("fill-opacity", 0.05)
                .attr("stroke", color)
                .attr("stroke-opacity", 0.2)
                .attr("stroke-dasharray", "3 3");
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

        g.append("g")
            .call(d3.axisLeft(y).ticks(5))
            .attr("color", "#ffffff66")
            .attr("font-size", "12px")
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
                    <div style="font-weight: bold; color: ${color};">${name}: ${Number(d[dataKey]).toFixed(6)}</div>
                `)
                    .style("top", `${cy - 120}px`)
                    .style("left", `${cx + 20}px`);
            }
        });

        svg.on("mouseleave", () => {
            focus.style("display", "none");
            tooltip.style("visibility", "hidden");
        });

    }, [data, dimensions, color, dataKey, gradientId, name, pivotPrice, channelWidth]);

    return (
        <div ref={containerRef} className="w-full h-full relative">
            <svg ref={svgRef} width={dimensions.width} height={dimensions.height} />
        </div>
    );
}
