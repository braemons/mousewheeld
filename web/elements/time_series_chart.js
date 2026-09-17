// SPDX-License-Identifier: AGPL-3.0-or-later
//
// One rolling time-series chart, as inline SVG.
//
// Rules it follows, and they are not taste:
//
//   * **One y-scale per chart, never two.** Position in cm and velocity in cm/s
//     on one pair of axes invites reading a crossing as an event when it is an
//     artefact of two arbitrary scales. Two measures means two charts.
//   * **A velocity chart always shows zero**, on the floor of the plot rather
//     than a padded distance below it: on that chart the zero line is the one
//     value that means something on its own — the animal stopped.
//   * **Two series get a legend and direct labels.** Identity is never carried
//     by colour alone.
//   * **A gap in the samples is drawn as a gap**, where the daemon reported
//     losing some. A line drawn straight through a hole is a claim about an
//     animal that nobody observed.
//
// The colours come from the shared stylesheet's custom properties: blue is what
// the host computed, orange is what the device reported.

const PAD_LEFT = 52;
const PAD_RIGHT = 12;
const PAD_TOP = 8;
const PAD_BOTTOM = 16;

/// Enough decimals to tell two neighbouring ticks apart, and no more.
export function decimalsFor(span) {
  if (span >= 100) return 0;
  if (span >= 10) return 1;
  if (span >= 1) return 2;
  return 3;
}

/// The value range to draw, padded away from zero only.
export function domainOf(values, includeZero) {
  let min = includeZero ? 0 : Infinity;
  let max = includeZero ? 0 : -Infinity;
  for (const value of values) {
    if (!Number.isFinite(value)) continue;
    min = Math.min(min, value);
    max = Math.max(max, value);
  }
  if (!Number.isFinite(min) || !Number.isFinite(max)) return { min: 0, max: 1 };
  if (max - min < 1e-9) return { min: min - 1, max: max + 1 };
  const pad = (max - min) * 0.08;
  return {
    min: includeZero && min >= 0 ? 0 : min - pad,
    max: includeZero && max <= 0 ? 0 : max + pad,
  };
}

/// Split a series at every gap the daemon reported, so a hole stays a hole.
///
/// `gapAfter` marks a point the stream did not continue from — the daemon said
/// it lost samples after it, or the producer was silent across it. The run ends
/// there and the next starts fresh, which draws as a break in the line.
function runsOf(points) {
  const runs = [];
  let current = [];
  for (const point of points) {
    current.push(point);
    if (point.gapAfter) {
      runs.push(current);
      current = [];
    }
  }
  if (current.length > 0) runs.push(current);
  return runs.filter((run) => run.length > 0);
}

/**
 * Draw one chart into a fresh `<svg>` and return it.
 *
 * `series` is `[{ name, color, pick }]`; `points` carry `{ t }` plus whatever
 * the pickers read, and optionally `gapAfter`.
 */
export function drawTimeSeries(svg, { points, series, windowS, now, includeZero, width, height }) {
  const plotWidth = width - PAD_LEFT - PAD_RIGHT;
  const plotHeight = height - PAD_TOP - PAD_BOTTOM;
  const values = [];
  for (const one of series) for (const point of points) values.push(one.pick(point));
  const domain = domainOf(values, includeZero);
  const t0 = now - windowS;

  const x = (t) => PAD_LEFT + Math.max(0, Math.min(1, (t - t0) / windowS)) * plotWidth;
  const y = (value) =>
    PAD_TOP + plotHeight - ((value - domain.min) / (domain.max - domain.min)) * plotHeight;

  const node = svg("svg", { width, height, viewBox: `0 0 ${width} ${height}` });
  const decimals = decimalsFor(domain.max - domain.min);

  // Direct labels, unless the lines have converged. Two labels stacked on one
  // another is worse than neither: nudging them apart detaches them from the
  // lines they name and reads as noise. Where the series meet — which for a
  // velocity pair is every time the animal stops — the legend carries identity
  // on its own.
  const newest = points[points.length - 1];
  const endYs = newest === undefined ? [] : series.map((one) => y(one.pick(newest)));
  const converged = endYs.some((value, index) =>
    endYs.some((other, otherIndex) => otherIndex !== index && Math.abs(value - other) < 11),
  );

  for (const tick of [domain.max, (domain.min + domain.max) / 2, domain.min]) {
    const isZero = includeZero && Math.abs(tick) < 1e-9;
    node.append(
      svg("line", {
        x1: PAD_LEFT,
        x2: width - PAD_RIGHT,
        y1: y(tick),
        y2: y(tick),
        stroke: isZero ? "var(--chart-zero)" : "var(--chart-grid)",
        "stroke-width": 1,
      }),
      svg("text", {
        x: PAD_LEFT - 6,
        y: y(tick) + 3,
        "text-anchor": "end",
        "font-size": 9,
        fill: "var(--muted)",
        text: tick.toFixed(decimals),
      }),
    );
  }

  for (const one of series) {
    for (const run of runsOf(points)) {
      if (run.length < 2) continue;
      const d = run
        .map((point, index) => `${index ? "L" : "M"}${x(point.t).toFixed(1)},${y(one.pick(point)).toFixed(1)}`)
        .join(" ");
      node.append(
        svg("path", {
          d,
          fill: "none",
          stroke: one.color,
          "stroke-width": 2,
          "stroke-linejoin": "round",
          "stroke-linecap": "round",
        }),
      );
    }
    if (newest === undefined) continue;
    node.append(
      svg("circle", {
        cx: x(newest.t),
        cy: y(one.pick(newest)),
        r: 3,
        fill: one.color,
        stroke: "var(--chart-surface)",
        "stroke-width": 2,
      }),
    );
    // Left of the marker, because the marker is already at the right edge and
    // the plot has no room to its right.
    if (series.length > 1 && !converged) {
      node.append(
        svg("text", {
          x: x(newest.t) - 7,
          y: Math.max(y(one.pick(newest)) - 6, PAD_TOP + 8),
          "text-anchor": "end",
          "font-size": 9,
          fill: "var(--muted)",
          text: one.name,
        }),
      );
    }
  }

  node.append(
    svg("text", {
      x: PAD_LEFT,
      y: height - 4,
      "font-size": 9,
      fill: "var(--muted)",
      text: `−${windowS} s`,
    }),
    svg("text", {
      x: width - PAD_RIGHT,
      y: height - 4,
      "text-anchor": "end",
      "font-size": 9,
      fill: "var(--muted)",
      text: "now",
    }),
  );
  return node;
}

/// Vertical rules for events — zone hits, marks — on a chart's own time base.
///
/// Annotations, not a series: they wear ink, never a series colour, so nobody
/// reads a hit as a third measurement.
export function drawEventRules(svg, { events, windowS, now, width, height }) {
  const plotWidth = width - PAD_LEFT - PAD_RIGHT;
  const t0 = now - windowS;
  const marks = [];
  for (const event of events) {
    if (event.t < t0 || event.t > now) continue;
    const x = PAD_LEFT + ((event.t - t0) / windowS) * plotWidth;
    marks.push(
      svg("line", {
        x1: x,
        x2: x,
        y1: PAD_TOP,
        y2: height - PAD_BOTTOM,
        stroke: "var(--panel-heading)",
        "stroke-width": 1,
        "stroke-dasharray": "3 2",
        opacity: 0.8,
      }),
      svg("text", {
        x: x + 3,
        y: PAD_TOP + 8,
        "font-size": 9,
        fill: "var(--panel-heading)",
        text: event.label,
      }),
    );
  }
  return marks;
}
