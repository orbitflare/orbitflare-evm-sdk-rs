use std::sync::Mutex;
use std::time::{Duration, Instant};

struct Endpoint {
    url: String,
    consecutive_failures: u32,
    available_after: Instant,
}

pub(crate) struct EndpointSet {
    inner: Mutex<Vec<Endpoint>>,
}

impl EndpointSet {
    pub fn new(primary: &str, fallbacks: &[String]) -> Self {
        let now = Instant::now();
        let mut endpoints = Vec::with_capacity(1 + fallbacks.len());
        endpoints.push(Endpoint {
            url: primary.to_string(),
            consecutive_failures: 0,
            available_after: now,
        });
        for fb in fallbacks {
            endpoints.push(Endpoint {
                url: fb.clone(),
                consecutive_failures: 0,
                available_after: now,
            });
        }
        Self {
            inner: Mutex::new(endpoints),
        }
    }

    pub fn get(&self, idx: usize) -> String {
        self.inner.lock().unwrap()[idx].url.clone()
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    pub fn pick(&self) -> usize {
        let endpoints = self.inner.lock().unwrap();
        let now = Instant::now();

        for (i, ep) in endpoints.iter().enumerate() {
            if now >= ep.available_after {
                return i;
            }
        }

        endpoints
            .iter()
            .enumerate()
            .min_by_key(|(_, ep)| ep.available_after)
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    pub fn mark_success(&self, idx: usize) {
        let mut endpoints = self.inner.lock().unwrap();
        if let Some(ep) = endpoints.get_mut(idx) {
            ep.consecutive_failures = 0;
            ep.available_after = Instant::now();
        }
    }

    pub fn mark_failure(&self, idx: usize) {
        let mut endpoints = self.inner.lock().unwrap();
        if let Some(ep) = endpoints.get_mut(idx) {
            ep.consecutive_failures += 1;
            let secs = (5u64 << ep.consecutive_failures.min(4)).min(60);
            ep.available_after = Instant::now() + Duration::from_secs(secs);
        }
    }
}
