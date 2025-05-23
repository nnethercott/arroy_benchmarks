# arroy benchmarks
A small repo to test if my features actually improve performance. Benchmarks done with criterion.

datacomp small vectors: https://meilisearch.notion.site/Movies-embeddings-1de3258859f54b799b7883882219d266

# checks
## binary heap vs median-based top k
Constructing a binary heap from n items and popping k elements costs O(n + klog(n)). Using a median-based strategy this cost goes down to O(n+klog(k)). Strategy from [this blog post](https://quickwit.io/blog/top-k-complexity).

<details>
  <summary>Sample bench</summary>

  ```bash
  heap/(n:100,k:10)
              time:   [1.3426 µs 1.3540 µs 1.3672 µs]
  median/(n:100,k:10)
              time:   [974.39 ns 984.91 ns 997.26 ns]
  heap/(n:100,k:100)
              time:   [4.5147 µs 4.5563 µs 4.5997 µs]
  median/(n:100,k:100)
              time:   [3.2994 µs 3.3830 µs 3.4817 µs]
  heap/(n:1000,k:10)
              time:   [9.5652 µs 9.7025 µs 9.8644 µs]
  median/(n:1000,k:10)
              time:   [2.4634 µs 2.4949 µs 2.5329 µs]
  heap/(n:1000,k:100)
              time:   [15.354 µs 15.577 µs 15.836 µs]
  median/(n:1000,k:100)
              time:   [12.392 µs 12.535 µs 12.714 µs]
  heap/(n:1000,k:1000)
              time:   [68.019 µs 68.696 µs 69.489 µs]
  median/(n:1000,k:1000)
              time:   [44.650 µs 45.091 µs 45.596 µs]
  heap/(n:10000,k:10)
              time:   [87.452 µs 88.466 µs 89.602 µs]
  median/(n:10000,k:10)
              time:   [9.0498 µs 9.2777 µs 9.5439 µs]
  heap/(n:10000,k:100)
              time:   [98.695 µs 100.90 µs 103.61 µs]
  median/(n:10000,k:100)
              time:   [27.534 µs 27.969 µs 28.473 µs]
  heap/(n:10000,k:1000)
              time:   [182.67 µs 184.45 µs 186.40 µs]
  median/(n:10000,k:1000)
                          time:   [131.45 µs 132.71 µs 134.14 µs]

  ```

</details>

## replace OrderedFloat package with byte-wise Ord on u32 trasmutes
I got this idea while reading [this article](https://ohadravid.github.io/posts/2025-05-rav1d-faster/#replace-field-wise-equality-with-byte-wise-equality-that-optimizes-better).

Essentially since distances are such that d(x,y)>=0 (it's one of their [core properties](https://en.wikipedia.org/wiki/Metric_space#Definition_and_illustration)) we can afford a cheaper comparison between non-negative f32s. As a u32 there's only a few special floats we need to watch out for; 
* inf which has all exponent bits set to 1's and 0's everywhere
* nan which has all exponent bits set to 1's and at least one non zero fraction bit set to 1

![alt-text](https://upload.wikimedia.org/wikipedia/commons/thumb/d/d2/Float_example.svg/885px-Float_example.svg.png)

This means that `f32::INFINITY.to_bits()<f32::NAN.to_bits()` ! But we don't really care since NaN's should be caught earlier before our sorting ops.

The ordered-float package implements `Ord` for floating point types, but when sorting distances (like when trying to find the top k elements) we don't need its full expressiveness - we can just cast the f32 to bits and compare directly. Doing it this way actually yields significant speedups. 

For this benchmark I just timed how long it took to build a binary heap from each wrapper type for varying input sizes.


<details>
  <summary>Sample bench</summary>

  ```bash
  OrderedFloat/n=10
             time:   [68.947 ns 70.406 ns 72.469 ns]
  NonNegativeOrderedFloat/n=10
             time:   [50.342 ns 50.838 ns 51.402 ns]
  OrderedFloat/n=100
             time:   [861.26 ns 877.75 ns 897.06 ns]
  NonNegativeOrderedFloat/n=100
             time:   [445.68 ns 452.93 ns 461.57 ns]
  OrderedFloat/n=1000
             time:   [8.9741 µs 9.1250 µs 9.2960 µs]
  NonNegativeOrderedFloat/n=1000
             time:   [3.8946 µs 3.9375 µs 3.9892 µs]
```

</details>

