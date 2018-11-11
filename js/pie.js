import * as d3 from "d3";
import { partition } from "lodash";

import { Interval } from "./remover";

window.d3 = d3;

function midAngle(d) {
  return d.startAngle + (d.endAngle - d.startAngle) / 2;
}

export function pieChart(labelRemover) {
  let data = [];
  let width = 100;
  let height = 100;

  const key = d => d.data.label;
  const color = d3
    .scaleOrdinal()
    .domain([
      "cactus",
      "belief",
      "ray",
      "room",
      "journey",
      "visitor",
      "pear",
      "hair",
      "advice",
      "shake",
      "afterthought",
      "disease",
      "branch",
      "surprise",
      "trade",
      "hydrant",
      "woman",
      "calendar",
      "measure",
      "rhythm",
      "gun",
      "front",
      "knee",
      "feast",
      "science",
      "wheel",
      "stitch",
      "powder"
    ])
    .range([
      "#98abc5",
      "#8a89a6",
      "#7b6888",
      "#6b486b",
      "#a05d56",
      "#d0743c",
      "#ff8c00"
    ]);

  const pie = d3
    .pie()
    .sort(null)
    .value(function(d) {
      return d.value;
    });

  const normDist = d3.randomNormal(500, 1000);

  function randomData() {
    const labels = color.domain();
    data = labels.map(function(label) {
      return { label: label, value: Math.floor(Math.max(normDist(), 1)) };
    });

    return draw;
  }
  function size(w, h) {
    width = Math.max(100, w);
    height = Math.max(100, h);
    return draw;
  }

  function draw(svgNode) {
    const svg = d3.select(svgNode);
    const radius = Math.min(width, height) / 2 - 20;
    const arc = d3
      .arc()
      .outerRadius(radius * 0.8)
      .innerRadius(radius * 0.4);

    const outerArc = d3
      .arc()
      .innerRadius(radius * 0.9)
      .outerRadius(radius * 0.9);

    const interval = d => {
      const [_x, y] = outerArc.centroid(d);
      const side = midAngle(d) < Math.PI ? 1 : -1;

      return {
        side,
        interval: new Interval(d.data.value, y, y + 36)
      };
    };
    const pieData = pie(data);
    const [left, right] = partition(pieData, x => interval(x).side === 1);

    const filteredLeft = labelRemover(left, x => interval(x).interval);
    const filteredRight = labelRemover(right, x => interval(x).interval);

    svg.attr("width", width).attr("height", height);

    const layers = svg
      .selectAll("g.layer")
      .data(["slices", "labels", "lines"], k => k);

    layers
      .enter()
      .append("g")
      .attr("class", d => `layer ${d}`)
      .merge(layers)
      .attr("transform", "translate(" + width / 2 + "," + height / 2 + ")");

    layers.exit().remove();

    const slice = svg
      .select(".slices")
      .selectAll("path.slice")
      .data(pieData, key);

    slice
      .enter()
      .append("path")
      .attr("class", "slice")
      .style("fill", function(d) {
        return color(d.data.label);
      })
      .merge(slice)
      .transition()
      .duration(1000)
      .attrTween("d", function(d) {
        this._current = this._current || d;
        var interpolate = d3.interpolate(this._current, d);
        this._current = interpolate(0);
        return function(t) {
          return arc(interpolate(t));
        };
      });

    slice.exit().remove();

    const labelData = filteredLeft.concat(filteredRight);

    text(radius, svg, labelData, outerArc);
    lines(radius, svg, labelData, arc, outerArc);
  }

  function text(radius, svg, textData, outerArc) {
    const text = svg
      .select(".labels")
      .selectAll("text")
      .data(textData, key);

    text
      .enter()
      .append("text")
      .attr("font-size", "24px")
      .attr("dy", ".35em")
      .text(function(d) {
        return d.data.label;
      })
      .merge(text)
      .transition()
      .duration(1000)
      .attrTween("transform", function(d) {
        this._current = this._current || d;
        var interpolate = d3.interpolate(this._current, d);
        this._current = interpolate(0);
        return function(t) {
          var d2 = interpolate(t);
          var pos = outerArc.centroid(d2);
          pos[0] = radius * (midAngle(d2) < Math.PI ? 1 : -1);
          return "translate(" + pos + ")";
        };
      })
      .styleTween("text-anchor", function(d) {
        this._current = this._current || d;
        var interpolate = d3.interpolate(this._current, d);
        this._current = interpolate(0);
        return function(t) {
          var d2 = interpolate(t);
          return midAngle(d2) < Math.PI ? "start" : "end";
        };
      });

    text.exit().remove();
  }
  function lines(radius, svg, textData, arc, outerArc) {
    const polyline = svg
      .select(".lines")
      .selectAll("polyline")
      .data(textData, key);

    polyline
      .enter()
      .append("polyline")
      .merge(polyline)
      .transition()
      .duration(1000)
      .attrTween("points", function(d) {
        this._current = this._current || d;
        var interpolate = d3.interpolate(this._current, d);
        this._current = interpolate(0);
        return function(t) {
          var d2 = interpolate(t);
          var pos = outerArc.centroid(d2);
          pos[0] = radius * 0.95 * (midAngle(d2) < Math.PI ? 1 : -1);
          return [arc.centroid(d2), outerArc.centroid(d2), pos];
        };
      });

    polyline.exit().remove();
  }

  draw.size = size;
  draw.randomData = randomData;
  draw.draw = draw;

  return draw;
}
