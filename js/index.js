import interact from "interactjs";
import pinch from "touch-pinch";
import throttle from "raf-throttle";
import * as d3 from "d3";
import { fromModule } from "./remover";
import { pieChart } from "./pie";

const stage = document.querySelector(".container");
const chart = document.querySelector(".chart");

const resizeDrag = interact(chart)
  .draggable({
    // onmove: window.dragMoveListener,
    restrict: {
      restriction: "parent",
      elementRect: { top: 0, left: 0, bottom: 1, right: 1 }
    }
  })
  .resizable({
    // resize from all edges and corners
    edges: { left: true, right: true, bottom: true, top: true },

    // keep the edges inside the parent
    restrictEdges: {
      outer: "parent",
      endOnly: true
    },

    // minimum size
    restrictSize: {
      min: { width: 200, height: 200 }
    },

    inertia: true
  });

function sizeInner(width, height) {
  return [width - 100, height - 64];
}
function sizeOutter(width, height) {
  return [width + 100, height + 64];
}

import("../crate/pkg").then(module => {
  const svgNode = document.querySelector("svg");
  const pie = pieChart(fromModule(module));

  pie
    .size(380, 250)
    .randomData()
    .draw(svgNode);

  let asp = 1.0;
  let pinching = false;

  const pinched = pinch(chart);

  pinched.on("start", function() {
    const [w, h] = pie.size();
    asp = h / w;
    pinching = true;
  });
  pinched.on("end", function() {
    pinching = false;
  });
  pinched.on(
    "change",
    throttle(function(dist, prev) {
      const delta = dist - prev;

      const [w, h] = pie.size();
      const width = w + delta;
      const height = width * asp;
      const [outterW, outterH] = sizeOutter(width, height);

      pie.size(width, height).draw(svgNode);

      chart.style.width = outterW + "px";
      chart.style.height = outterH + "px";
    })
  );

  resizeDrag.on(
    "resizemove",
    throttle(function(event) {
      let target = event.target,
        x = parseFloat(target.getAttribute("data-x")) || 0,
        y = parseFloat(target.getAttribute("data-y")) || 0,
        width = event.rect.width,
        height = event.rect.height;

      // update the element's style
      target.style.width = width + "px";
      target.style.height = height + "px";

      // translate when resizing from top or left edges
      x += event.deltaRect.left;
      y += event.deltaRect.top;

      target.style.webkitTransform = target.style.transform =
        "translate(" + x + "px," + y + "px)";

      target.setAttribute("data-x", x);
      target.setAttribute("data-y", y);

      pie.size(...sizeInner(width, height)).draw(svgNode);
    })
  );
  resizeDrag.on(
    "dragmove",
    throttle(function(event) {
      if (pinching) {
        return;
      }

      let target = event.target,
        x = parseFloat(target.getAttribute("data-x")) || 0,
        y = parseFloat(target.getAttribute("data-y")) || 0;

      x += event.dx;
      y += event.dy;

      target.style.webkitTransform = target.style.transform =
        "translate(" + x + "px," + y + "px)";

      target.setAttribute("data-x", x);
      target.setAttribute("data-y", y);
    })
  );

  d3.select(".random-data").on("click", function() {
    pie.randomData().draw(svgNode);
  });
});
