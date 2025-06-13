# arroy benchmarks
A small repo to test if my features actually improve performance. Benchmarks done with criterion.



## install
To build the arroy index i used the [datacomp small vectors](https://meilisearch.notion.site/Movies-embeddings-1de3258859f54b799b7883882219d26). Then unzipping and building the index can be done with:
```
 gzip -d < ~/Downloads/vectors.txt.gzip > ./assets/vectors.txt
 cargo run --bin import -- --n-trees 20
```

To run the benchmarks:
```bash
cargo bench
```


# ideas
## binary heap vs median-based top k
Constructing a binary heap from n items and popping k elements costs O(n + klog(n)). Using a median-based strategy this cost goes down to O(n+klog(k)). Strategy from [this blog post](https://quickwit.io/blog/top-k-complexity).

  In the table below we list the times like "median-based"/"binary heap"

| n \ k | k=10 | k=100 | k=1000 |
|-------|------|-------|-------|
| **n=100** | 0.985 µs/1.354 µs| 3.383 µs/4.556 µs| - |
| **n=1000** | 2.495 µs/9.703 µs| 12.535 µs/15.577 µs| 45.091 µs/68.696 µs|
| **n=10000** | 9.278 µs/88.466 µs | 27.969 µs/100.90 µs| 132.71 µs/184.45 µs|


## replace OrderedFloat package with byte-wise Ord on u32 trasmutes
I got this idea while reading [this article](https://ohadravid.github.io/posts/2025-05-rav1d-faster/#replace-field-wise-equality-with-byte-wise-equality-that-optimizes-better).

Essentially since distances are such that d(x,y)>=0 (it's one of their [core properties](https://en.wikipedia.org/wiki/Metric_space#Definition_and_illustration)) we can afford a cheaper comparison between non-negative f32s. As a u32 there's only a few special floats we need to watch out for;
* inf which has all exponent bits set to 1's and 0's everywhere
* nan which has all exponent bits set to 1's and at least one non zero fraction bit set to 1

![alt-text](https://upload.wikimedia.org/wikipedia/commons/thumb/d/d2/Float_example.svg/885px-Float_example.svg.png)

This means that `f32::INFINITY.to_bits()<f32::NAN.to_bits()` ! But we don't really care since NaN's should be caught earlier before our sorting ops.

The ordered-float package implements `Ord` for floating point types, but when sorting distances (like when trying to find the top k elements) we don't need its full expressiveness - we can just cast the f32 to bits and compare directly. Doing it this way actually yields significant speedups.

For this benchmark I just timed how long it took to build a binary heap from each wrapper type for varying input sizes.


| n | OrderedFloat | NonNegativeOrderedFloat |
|---|--------------|-------------------------|
| **n=10** | 70.406 ns | 50.838 ns |
| **n=100** | 877.75 ns | 452.93 ns |
| **n=1000** | 9.1250 µs | 3.9375 µs |



# relevancy
```bash
cargo run --bin recall -- --n-trees 1 --n-vecs 10000 --dataset movies
```


# ablation study n-trees
To auto-provision the number of trees we're looking for a function f(dim, n) mapping embedding dimension and dataset size to a recommendation on the number of trees giving "adequate" performance
- `n_trees`  large will always give the best performance but has a drawback in longer build times and longer search times

## 1k
- trees: 1, build time: 43.03ms, ndcg@100: 0.76
- trees: 2, build time: 74.06ms, ndcg@100: 0.77
- trees: 8, build time: 145.97ms, ndcg@100: 0.92
- trees: 32, build time: 615.67ms, ndcg@100: 0.99
- trees: 64, build time: 1.02s, ndcg@100: 0.99
- trees: 128, build time: 2.04s, ndcg@100: 1.0

## 5k
- trees: 1, build time: 262.36ms, ndcg@100: 0.54
- trees: 2, build time: 74.06ms, ndcg@100: 0.62
- trees: 8, build time: 145.97ms, ndcg@100: 0.66
- trees: 32, build time: 615.67ms, ndcg@100: 0.87
- trees: 64, build time: 1.02s, ndcg@100: 0.91
- trees: 128, build time: 2.04s, ndcg@100: 95

## 10k
- trees: 1, build time: 903.15ms, ndcg@100: 0.35
- trees: 2, build time: 1.62s, ndcg@100: 0.46
- trees: 8, build time: 5.53s, ndcg@100: 0.57
- trees: 32, build time: 24.46s, ndcg@100: 0.7
- trees: 64, build time: 50.12s, ndcg@100: 0.8
- trees: 128, build time: 92.26s, ndcg@100: 0.90

## 50k
- trees: 1, build time: 6.50s, ndcg@100: 0.39
- trees: 32, build time: 615.67ms, ndcg@100: 0.62
- trees: 128, build time: 2.04s, ndcg@100: 0.79

## 100k
- trees: 32, build time: 615.67ms, ndcg@100: 0.66
- trees: 128, build time: 1446.39, ndcg@100: 0.81


## observations
- build time goes up linearly
