# arroy benchmarks
A small repo to test if my features actually improve perfs

vectors from https://meilisearch.notion.site/Movies-embeddings-1de3258859f54b799b7883882219d266

## vector store benchmark results
median top k
```
Datacomp small - 12799999 vectors of 768 dimensions
10000 vectors are used for this measure and 18446744073709551615B of memory
Recall tested is:   [   1,   10,   20,   50,  100,  500]
Starting indexing process
indexing: 50.38s, size: 132.02 MiB, indexed in 1x
[arroy]  Cosine x1: [1.00, 0.81, 0.83, 0.89, 0.92, 0.97], searched for: 6.89s, searched in 100.00%
[arroy]  Cosine x1: [1.00, 0.84, 0.88, 0.92, 0.95, 0.99], searched for: 6.01s, searched in 50.00%
[arroy]  Cosine x1: [1.00, 0.89, 0.91, 0.94, 0.96, 0.99], searched for: 6.96s, searched in 25.00%
[arroy]  Cosine x1: [1.00, 0.92, 0.93, 0.95, 0.96, 1.00], searched for: 8.08s, searched in 15.00%
[arroy]  Cosine x1: [1.00, 0.93, 0.94, 0.96, 0.97, 1.00], searched for: 9.47s, searched in 10.00%
[arroy]  Cosine x1: [1.00, 0.94, 0.94, 0.96, 0.97, 1.00], searched for: 9.78s, searched in 8.00%
[arroy]  Cosine x1: [1.00, 0.94, 0.95, 0.96, 0.97, 1.00], searched for: 11.23s, searched in 6.00%
[arroy]  Cosine x1: [1.00, 0.96, 0.96, 0.98, 1.00, 0.40], searched for: 14.68s, searched in 2.00%
[arroy]  Cosine x1: [1.00, 0.97, 0.98, 0.99, 1.00, 0.20], searched for: 17.92s, searched in 1.00%
[arroy]  Cosine x3: [1.00, 0.88, 0.91, 0.95, 0.96, 1.00], searched for: 12.81s, searched in 100.00%
[arroy]  Cosine x3: [1.00, 0.92, 0.94, 0.96, 0.97, 1.00], searched for: 11.24s, searched in 50.00%
[arroy]  Cosine x3: [1.00, 0.94, 0.95, 0.97, 0.98, 1.00], searched for: 15.47s, searched in 25.00%
[arroy]  Cosine x3: [1.00, 0.95, 0.96, 0.97, 0.99, 1.00], searched for: 26.28s, searched in 15.00%
[arroy]  Cosine x3: [1.00, 0.96, 0.96, 0.98, 0.99, 1.00], searched for: 27.35s, searched in 10.00%
[arroy]  Cosine x3: [1.00, 0.96, 0.96, 0.98, 1.00, 1.00], searched for: 25.56s, searched in 8.00%
[arroy]  Cosine x3: [1.00, 0.96, 0.96, 0.99, 1.00, 1.00], searched for: 27.04s, searched in 6.00%
[arroy]  Cosine x3: [1.00, 0.98, 0.99, 1.00, 1.00, 0.40], searched for: 27.09s, searched in 2.00%
[arroy]  Cosine x3: [1.00, 0.99, 1.00, 1.00, 1.00, 0.20], searched for: 29.80s, searched in 1.00%
```

main 
```
Datacomp small - 12799999 vectors of 768 dimensions
10000 vectors are used for this measure and 18446744073709551615B of memory
Recall tested is:   [   1,   10,   20,   50,  100,  500]
Starting indexing process
indexing: 56.79s, size: 132.02 MiB, indexed in 1x
[arroy]  Cosine x1: [0.95, 0.81, 0.81, 0.87, 0.92, 0.97], searched for: 7.40s, searched in 100.00%
[arroy]  Cosine x1: [0.95, 0.83, 0.85, 0.90, 0.95, 0.99], searched for: 6.49s, searched in 50.00%
[arroy]  Cosine x1: [0.95, 0.88, 0.89, 0.92, 0.95, 0.99], searched for: 8.14s, searched in 25.00%
[arroy]  Cosine x1: [0.95, 0.91, 0.90, 0.93, 0.96, 1.00], searched for: 9.23s, searched in 15.00%
[arroy]  Cosine x1: [0.95, 0.92, 0.91, 0.93, 0.96, 1.00], searched for: 10.03s, searched in 10.00%
[arroy]  Cosine x1: [0.95, 0.93, 0.91, 0.93, 0.96, 1.00], searched for: 12.47s, searched in 8.00%
[arroy]  Cosine x1: [0.95, 0.93, 0.92, 0.94, 0.97, 1.00], searched for: 13.33s, searched in 6.00%
[arroy]  Cosine x1: [0.95, 0.95, 0.93, 0.96, 0.99, 0.40], searched for: 17.59s, searched in 2.00%
[arroy]  Cosine x1: [0.95, 0.96, 0.95, 0.97, 1.00, 0.20], searched for: 19.84s, searched in 1.00%
[arroy]  Cosine x3: [0.95, 0.87, 0.88, 0.92, 0.96, 1.00], searched for: 12.92s, searched in 100.00%
[arroy]  Cosine x3: [0.95, 0.91, 0.91, 0.94, 0.97, 1.00], searched for: 11.50s, searched in 50.00%
[arroy]  Cosine x3: [0.95, 0.93, 0.92, 0.94, 0.98, 1.00], searched for: 16.68s, searched in 25.00%
[arroy]  Cosine x3: [0.95, 0.94, 0.93, 0.95, 0.98, 1.00], searched for: 27.62s, searched in 15.00%
[arroy]  Cosine x3: [0.95, 0.95, 0.93, 0.95, 0.99, 1.00], searched for: 27.65s, searched in 10.00%
[arroy]  Cosine x3: [0.95, 0.95, 0.93, 0.96, 0.99, 1.00], searched for: 28.35s, searched in 8.00%
[arroy]  Cosine x3: [0.95, 0.95, 0.94, 0.96, 0.99, 1.00], searched for: 27.58s, searched in 6.00%
[arroy]  Cosine x3: [0.95, 0.97, 0.96, 0.97, 1.00, 0.40], searched for: 33.41s, searched in 2.00%
[arroy]  Cosine x3: [0.95, 0.98, 0.97, 0.97, 1.00, 0.20], searched for: 32.48s, searched in 1.00%
```

