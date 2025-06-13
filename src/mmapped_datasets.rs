use std::fs::File;
use std::marker::PhantomData;
use std::mem;
use std::sync::Arc;

use anyhow::Context;
use bytemuck::{AnyBitPattern, PodCastError};
use memmap2::Mmap;

#[derive(Debug, Clone)]
pub struct MatLEView<T> {
    name: &'static str,
    mmap: Arc<Mmap>,
    dimensions: usize,
    _marker: PhantomData<T>,
}

impl<T: AnyBitPattern> MatLEView<T> {
    pub fn new(name: &'static str, path: &str, dimensions: usize) -> MatLEView<T> {
        let file = File::open(path).with_context(|| format!("while opening {path}")).unwrap();
        let mmap = unsafe { Mmap::map(&file).unwrap() };

        assert!(mmap.len() != 0, "The file is empty");
        assert!((mmap.len() / mem::size_of::<T>()) % dimensions == 0);
        MatLEView { name, mmap: Arc::new(mmap), dimensions, _marker: PhantomData }
    }

    pub fn header(&self) {
        println!(
            "{} - {} vectors of \x1b[1m{}\x1b[0m dimensions",
            self.name,
            self.len(),
            self.dimensions
        );
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        ((self.mmap.len() / mem::size_of::<T>()) / self.dimensions) - 1
    }

    pub fn get(&self, index: usize) -> Option<Result<&[T], PodCastError>> {
        let tsize = mem::size_of::<T>();
        if (index * self.dimensions + self.dimensions) * tsize < self.mmap.len() {
            let start = index * self.dimensions;
            let bytes = &self.mmap[start * tsize..(start + self.dimensions) * tsize];
            match bytemuck::try_cast_slice::<u8, T>(bytes) {
                Ok(slice) => Some(Ok(slice)),
                Err(e) => Some(Err(e)),
            }
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &[T]> {
        (0..self.len()).map(|i| self.get(i).unwrap().unwrap())
    }

    pub fn get_all(&self) -> Vec<&[T]> {
        self.iter().collect()
    }
}

impl<T> PartialEq for MatLEView<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(other.name)
    }
}

impl<T> Eq for MatLEView<T> {}

impl<T> PartialOrd for MatLEView<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for MatLEView<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(other.name)
    }
}

pub const DATACOMP_SMALL: &str = "assets/datacomp-small.mat";
pub const DATACOMP_SMALL_DIMENSIONS: usize = 768;
pub fn datacomp_small() -> MatLEView<f32> {
    MatLEView::new("Hackernews top posts", DATACOMP_SMALL, DATACOMP_SMALL_DIMENSIONS)
}

pub const HN_TOP_POSTS_PATH: &str = "assets/hn-top-posts.mat";
pub const HN_TOP_POSTS_DIMENSIONS: usize = 1024;
pub fn hn_top_posts() -> MatLEView<f32> {
    MatLEView::new("Hackernews top posts", HN_TOP_POSTS_PATH, HN_TOP_POSTS_DIMENSIONS)
}

pub const HN_POSTS_PATH: &str = "assets/hn-posts.mat";
pub const HN_POSTS_DIMENSIONS: usize = 512;
pub fn hn_posts() -> MatLEView<f32> {
    MatLEView::new("Hackernews posts", HN_POSTS_PATH, HN_POSTS_DIMENSIONS)
}

pub const DB_PEDIA_OPENAI_TEXT_EMBEDDING_3_LARGE_PATH: &str =
    "assets/db-pedia-OpenAI-text-embedding-3-large.mat";
pub const DB_PEDIA_OPENAI_TEXT_EMBEDDING_3_LARGE_DIMENSIONS: usize = 3072;
// pub fn hn_posts() -> MatLEView<f32> {
//     MatLEView::new("Hackernews posts", "assets/hn-posts.mat", 512)
// }

pub const DB_PEDIA_OPENAI_TEXT_EMBEDDING_ADA_002_PATH: &str =
    "assets/db-pedia-OpenAI-text-embedding-3-large.mat";
pub const DB_PEDIA_OPENAI_TEXT_EMBEDDING_ADA_002_DIMENSIONS: usize = 1536;
// pub fn hn_posts() -> MatLEView<f32> {
//     MatLEView::new("Hackernews posts", "assets/hn-posts.mat", 512)
// }
