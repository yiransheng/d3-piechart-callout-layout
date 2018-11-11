#[macro_use]
extern crate cfg_if;
extern crate int_hash;
extern crate pathfinding;
extern crate wasm_bindgen;
extern crate web_sys;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate smallvec;

cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        use console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        fn set_panic_hook() {}
    }
}

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate core as std;
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

use wasm_bindgen::prelude::*;

use self::Ordering::*;
use std::cmp::Ordering;

use int_hash::IntHashSet;
use pathfinding::directed::topological_sort::topological_sort;

macro_rules! console_log {
    ($($t:tt)*) => {{
        web_sys::console::log_1(&( &format_args!($($t)*).to_string().into()) )
    }};
}

#[wasm_bindgen]
pub fn remove_overlapping(inputs: &[f64], output: &mut [u8]) {
    let n = inputs.len();
    let mut xs: Vec<Interval> = Vec::with_capacity(n / 3);

    for i in (0..n).into_iter().step_by(3) {
        let interval = Interval::from_f64_slice(&inputs[i..i + 3]);
        xs.push(interval.unwrap());
    }

    let g = build_graph(&xs);

    // clear
    for b in output.iter_mut() {
        *b = 0;
    }

    for index in g.longest_path() {
        output[index] = u8::max_value();
    }
}

trait Weighted: PartialOrd {
    fn weight(&self) -> u32;
}

#[derive(Debug)]
pub struct Interval {
    lower: f64,
    upper: f64,
    weight: u32,
}
impl PartialEq for Interval {
    fn eq(&self, other: &Self) -> bool {
        self.lower == other.lower && self.upper == other.upper && self.weight == other.weight
    }
}

impl PartialOrd for Interval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.upper.partial_cmp(&other.lower) {
            Some(Less) | Some(Equal) => return Some(Less),
            _ => {}
        };
        match other.upper.partial_cmp(&self.lower) {
            Some(Less) | Some(Equal) => return Some(Greater),
            _ => {}
        };

        None
    }
}
impl Weighted for Interval {
    fn weight(&self) -> u32 {
        self.weight
    }
}
impl<W: Weighted> Weighted for Box<W> {
    fn weight(&self) -> u32 {
        (**self).weight()
    }
}
impl Interval {
    pub fn new(weight: u32, lower: f64, upper: f64) -> Self {
        match lower.partial_cmp(&upper) {
            Some(Greater) => Interval {
                lower: upper,
                upper: lower,
                weight,
            },
            Some(Less) => Interval {
                lower,
                upper,
                weight,
            },
            _ => Interval {
                weight: 0,
                lower: ::std::f64::NAN,
                upper: ::std::f64::NAN,
            },
        }
    }
    pub fn from_f64_slice(slice: &[f64]) -> Option<Self> {
        if slice.len() != 3 {
            None
        } else {
            let lower = slice[0];
            let upper = slice[1];
            let weight = slice[2];
            Some(Self::new(weight as u32, lower, upper))
        }
    }
}

fn build_graph<W: Weighted>(xs: &[W]) -> Graph {
    let n = xs.len();

    let start_node = Node {
        predecessors: IntHashSet::default(),
        weight: 0,
        dist: 0,
        path_predecessor: None,
    };
    let g_nodes = (0..n).map(|i| {
        let x = &xs[i];
        let mut predecessors: IntHashSet<usize> = IntHashSet::default();
        predecessors.insert(0);
        predecessors.extend(
            (0..n)
                .filter(|j| match x.partial_cmp(&xs[*j]) {
                    Some(Greater) => true,
                    _ => false,
                })
                .map(|j| j + 1),
        );
        Node {
            predecessors,
            weight: x.weight(),
            dist: u32::min_value(),
            path_predecessor: None,
        }
    });
    let end_node = Node {
        predecessors: (1..=n).collect(),
        weight: 0,
        dist: u32::max_value(),
        path_predecessor: None,
    };

    let mut nodes = Vec::with_capacity(n + 2);
    nodes.push(start_node);
    nodes.extend(g_nodes);
    nodes.push(end_node);

    Graph { nodes }
}

struct Graph {
    nodes: Vec<Node>,
}

struct Node {
    predecessors: IntHashSet<usize>,
    weight: u32,
    dist: u32,
    path_predecessor: Option<usize>,
}

impl Graph {
    fn longest_path(mut self) -> Vec<usize> {
        debug_assert!(self.nodes.len() >= 2);

        let n = self.nodes.len();

        if n == 2 {
            return vec![];
        }
        if n == 3 {
            return vec![0];
        }

        let node_ids: Vec<usize> = (0..n).collect();
        let maybe_sorted = topological_sort(&node_ids, |node_id| {
            let node = &self.nodes[*node_id];
            node.predecessors.iter().map(|id| *id)
        });

        let mut sorted;
        match maybe_sorted {
            Ok(s) => {
                sorted = s;
            }
            _ => return vec![],
        }

        sorted.reverse();

        // dynamic programming loop
        for i in 0..n {
            self.mark_longest_path_for(sorted[i]);
        }
        let mut path = vec![];
        let mut node_id = n - 1;

        while let Some(path_predecessor) = self.nodes[node_id].path_predecessor {
            // we added dummy start/end node, ignore them for result path
            // and convert graph node index back to index in original input
            if node_id < n - 1 {
                path.push(node_id - 1);
            }
            node_id = path_predecessor;
        }
        path.reverse();

        path
    }

    fn mark_longest_path_for(&mut self, node_id: usize) {
        if node_id == 0 {
            self.nodes[0].dist = 0;
            self.nodes[0].path_predecessor = None;
            return;
        }

        let pred_dist;
        let path_predecessor;
        {
            let (i, pred_node) = self
                .predecessors(node_id)
                .max_by_key(|(_, node)| node.dist)
                .unwrap();

            pred_dist = pred_node.dist;
            path_predecessor = i;
        }

        let node = &mut self.nodes[node_id];
        let weight = node.weight;

        node.dist = pred_dist + weight;
        node.path_predecessor = Some(path_predecessor);
    }
    fn predecessors<'a>(&'a self, node_id: usize) -> impl Iterator<Item = (usize, &'a Node)> {
        self.nodes[node_id]
            .predecessors
            .iter()
            .map(move |pid| (*pid, &self.nodes[*pid]))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::hash::Hash;

    use int_hash::IntHasher;
    use quickcheck::{Arbitrary, Gen};
    use smallvec::SmallVec;

    use super::*;

    #[derive(Debug, Copy, Hash, Clone, Eq, PartialEq)]
    struct Span {
        lower: u32,
        upper: u32,
    }
    #[derive(Debug, Copy, Hash, Clone, Eq, PartialEq)]
    struct WeightedSpan {
        span: Span,
        weight: u32,
    }
    #[derive(Debug, Clone, Eq, PartialEq)]
    struct SmallSpans<S: Clone> {
        spans: SmallVec<[S; 16]>,
    }

    impl Span {
        fn new(lower: u32, upper: u32) -> Self {
            assert!(upper > lower);
            Span { lower, upper }
        }
    }
    impl WeightedSpan {
        fn new(weight: u32, lower: u32, upper: u32) -> Self {
            WeightedSpan {
                weight,
                span: Span::new(lower, upper),
            }
        }
    }
    impl SmallSpans<WeightedSpan> {
        fn as_slice(&self) -> &[WeightedSpan] {
            &self.spans
        }
        fn subset(&self, mask: u16) -> Self {
            let mut spans = SmallVec::new();

            let mut index_mask: u16 = 0b1000_0000_0000_0000;
            let mut index: usize = 0;

            loop {
                if index >= 16 {
                    break;
                }
                if mask & index_mask != 0 {
                    spans.push(self.spans[index]);
                }
                index += 1;
                index_mask >>= 1;
            }

            SmallSpans { spans }
        }
    }

    impl PartialOrd for Span {
        fn partial_cmp(&self, other: &Span) -> Option<Ordering> {
            if other.lower >= self.upper {
                Some(Less)
            } else if self.lower >= other.upper {
                Some(Greater)
            } else {
                None
            }
        }
    }
    impl Weighted for Span {
        fn weight(&self) -> u32 {
            1
        }
    }
    impl Weighted for WeightedSpan {
        fn weight(&self) -> u32 {
            self.weight
        }
    }
    impl PartialOrd for WeightedSpan {
        fn partial_cmp(&self, other: &WeightedSpan) -> Option<Ordering> {
            self.span.partial_cmp(&other.span)
        }
    }
    impl Arbitrary for Span {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let lower = u32::arbitrary(g) % 256;
            let size = 1 + u32::arbitrary(g) % 32;

            Span {
                lower,
                upper: lower + size,
            }
        }
    }
    impl Arbitrary for WeightedSpan {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let span = Span::arbitrary(g);
            let weight = u32::arbitrary(g) % 256 + 1;
            WeightedSpan { span, weight }
        }
    }
    impl Arbitrary for SmallSpans<WeightedSpan> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut spans = SmallVec::new();
            for _ in 0..16 {
                let span = WeightedSpan::arbitrary(g);
                spans.push(span);
            }
            SmallSpans { spans }
        }
    }

    #[test]
    fn test_non_overlapping() {
        let spans = vec![
            Span::new(0, 2),
            Span::new(2, 4),
            Span::new(4, 6),
            Span::new(6, 8),
        ];
        let g = build_graph(&spans);
        let path = g.longest_path();

        assert_eq!(&path, &[0, 1, 2, 3]);
    }

    #[test]
    fn test_simple_overlapping() {
        let spans = vec![
            Span::new(0, 2),
            Span::new(1, 3),
            Span::new(2, 4),
            Span::new(3, 5),
            Span::new(4, 6),
        ];
        let g = build_graph(&spans);
        let path = g.longest_path();

        assert_eq!(&path, &[0, 2, 4]);
    }
    #[test]
    fn test_simple_weighted() {
        let spans = vec![
            WeightedSpan::new(1, 0, 2),
            WeightedSpan::new(10, 1, 3),
            WeightedSpan::new(1, 2, 4),
        ];
        let g = build_graph(&spans);
        let path = g.longest_path();

        assert_eq!(&path, &[1]);
    }

    quickcheck! {
        fn removes_overlapping_spans(spans: Vec<WeightedSpan>) -> bool {
            let g = build_graph(&spans);
            let path = g.longest_path();
            let retained_spans: Vec<_> =
                path.into_iter().map(|i| spans[i]).collect();

            !path_overlaps(&retained_spans)
        }
        fn check_correctness(spans: SmallSpans<WeightedSpan>) -> bool {
            // this takes a minute to run
            let g = build_graph(spans.as_slice());
            let path = g.longest_path();
            let retained_spans: Vec<_> =
                path.into_iter().map(|i| spans.spans[i]).collect();

            let solution = solve_brute(spans);

            let score_1 = score(retained_spans.as_slice());
            let score_2 = score(solution.as_slice());

            score_1 == score_2
        }
    }

    fn solve_brute(spans: SmallSpans<WeightedSpan>) -> SmallSpans<WeightedSpan> {
        (0..u16::max_value())
            .map(|mask| spans.subset(mask))
            .filter(|subset| !path_overlaps(subset.as_slice()))
            .max_by_key(|subset| score(subset.as_slice()))
            .unwrap_or(SmallSpans {
                spans: SmallVec::new(),
            })
    }

    // for a list of partial ordered items, check if it is a total ordered
    // chain
    fn path_overlaps<W: Weighted>(xs: &[W]) -> bool {
        let n = xs.len();
        if n < 2 {
            return false;
        }
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                match xs[i].partial_cmp(&xs[j]) {
                    Some(_) => {}
                    _ => return true,
                }
            }
        }
        false
    }
    fn score<W: Weighted>(xs: &[W]) -> u32 {
        xs.into_iter().map(|s| s.weight()).sum()
    }
}
