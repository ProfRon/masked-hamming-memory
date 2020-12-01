# `Masked Hamming Memory`

A rust crate to implement Bernd Klauer's MHD (Masked Hamming Distance) Associative Memory (more or less) in Rust -- building on Huonw's excellent "hamming" crate.

## Inspiration

Huonw's repo is at <https://github.com/huonw/hamming>.

Huonw's original "Readme" was as follows:

> [![Build Status](https://travis-ci.org/huonw/hamming.png)](https://travis-ci.org/huonw/hamming) [![codecov](https://codecov.io/gh/huonw/hamming/branch/master/graph/badge.svg)](https://codecov.io/gh/huonw/hamming)
>
> A crate to compute the
> [Hamming weight](https://en.wikipedia.org/wiki/Hamming_weight) of a
> vector and the
> [Hamming distance](https://en.wikipedia.org/wiki/Hamming_distance)
> between two efficiently. This supports `no_std` environments.
>
> [Documentation](http://docs.rs/hamming), [crates.io](https://crates.io/crates/hamming).
> 

## Literature to Klauer's MHD:  

<pre>
@article{klauer_pen-based_1993,
	title = {Pen-based recognizing of handprinted characters},
	volume = {38},
	issn = {0165-6074},
	url = {http://www.sciencedirect.com/science/article/pii/016560749390230I},
	doi = {10.1016/0165-6074(93)90230-I},
	series = {Proceedings Euromicro 93 Open System Design: Hardware, Software and Applications},
	pages = {803--809},
	number = {1},
	journaltitle = {Microprocessing and Microprogramming},
	shortjournal = {Microprocessing and Microprogramming},
	author = {Klauer, Bernd and Waldschmidt, Klaus},
	urldate = {2020-12-01},
	date = {1993-09-01},
	langid = {english},
}

@inproceedings{klauer_object-oriented_1993,
	location = {Berlin, Heidelberg},
	title = {An object-oriented pen-based recognizer for handprinted characters},
	isbn = {978-3-540-47980-2},
	doi = {10.1007/3-540-57233-3_78},
	series = {Lecture Notes in Computer Science},
	pages = {586--593},
	booktitle = {Computer Analysis of Images and Patterns},
	publisher = {Springer},
	author = {Klauer, Bernd and Waldschmidt, Klaus},
	editor = {Chetverikov, Dmitry and Kropatsch, Walter G.},
	date = {1993},
	langid = {english},
}
</pre>

See also Ronald Moore's _Diplomarbeit_, if you can find a copy anywhere... ;-)

<pre>
@thesis{moore_ronald_charles_spracherkennung_1994,
	location = {Frankfurt am Main, Germany},
	title = {Spracherkennung nmit der {MHD} Methode},
	pagetotal = {152},
	institution = {Johann Wolfgang Goethe University},
	type = {Diplomarbeit},
	author = {Moore, Ronald Charles},
	date = {1994-06}
}
</pre>
