unique-state-disjoints
===

This is a project that spawned out of the widely known fun-fact that [Ohio is
the only state to share no letters with the word
mackerel](https://www.google.es/search?hl=en&q=ohio+mackerel). I was intrigued
by this fact and wanted to try to find all of the possible unique combinations
of words and states (referred to as
[disjoints](https://en.wikipedia.org/wiki/Disjoint_sets)). I have also recently
been trying to learn the [Rust programming
language](https://www.rust-lang.org/), and I thought that this project would
make a great pairing of the two ideas.

The List
---

For the most recent list, (It shouldn't really change very often) open
[disjoints.txt](disjoints.txt). For any curious readers, the Ohio, Mackerel
pairing can be seen by visiting [line 24184](disjoints.txt#L24184). There are
way more of these pairings than I thought there would be.

Use
---

If you're not inclined to belive the published lists, you can generate them
yourself using this tool.

	$ git clone https://github.com/natemara/unique-state-disjoints
	$ cd unique-state-disjoints
	$ cargo run --release -j4
