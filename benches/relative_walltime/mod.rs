use criterion::measurement::{Measurement, ValueFormatter, WallTime};
use std::time::{Duration, Instant};

// RelativeWallTime offsets all measurements by a provided d
pub struct RelativeWallTime {
    offset: Duration,
    walltime: WallTime,
}

impl RelativeWallTime {
    pub fn new<T>(baselinefn: fn() -> Result<T, failure::Error>) -> Result<Self, failure::Error> {
        Ok(RelativeWallTime {
            walltime: WallTime {},
            offset: Self::measure_best(baselinefn)?,
        })
    }

    pub fn measure_avg<T>(
        baselinefn: fn() -> Result<T, failure::Error>,
    ) -> Result<Duration, failure::Error> {
        println!("creating baseline...");
        // warm up
        baselinefn()?;

        let instant = Instant::now();
        baselinefn()?;
        // attempt to fill 5 seconds profiling.
        // should prolly check overflow here.
        let elapsed = instant.elapsed();
        let count = (Duration::new(5, 0).as_secs_f64() / elapsed.as_secs_f64()).round() as u64;
        println!(
            "  initial took {}ms, doing {} runs",
            elapsed.as_millis(),
            count
        );

        let mut sum: Duration = Duration::new(0, 0);

        for _ in 0..count {
            let instant = Instant::now();
            baselinefn()?;
            sum += instant.elapsed();
        }
        let res = Duration::from_secs_f64(sum.as_secs_f64() / (count as f64));

        println!("finished baseline of {} ms!", res.as_millis());
        Ok(res)
    }

    pub fn measure_best<T>(
        baselinefn: fn() -> Result<T, failure::Error>,
    ) -> Result<Duration, failure::Error> {
        println!("creating baseline...");
        // warm up
        baselinefn()?;

        let instant = Instant::now();
        baselinefn()?;
        // attempt to fill 5 seconds profiling.
        // should prolly check overflow here.
        let elapsed = instant.elapsed();
        let count = (Duration::new(5, 0).as_secs_f64() / elapsed.as_secs_f64()).round() as u64;
        println!(
            "  initial took {}ms, doing {} runs",
            elapsed.as_millis(),
            count
        );

        let mut sum: Duration = Duration::new(0, 0);
        let mut best: Duration = elapsed;

        for _ in 0..count {
            let instant = Instant::now();
            baselinefn()?;
            let elapsed = instant.elapsed();
            if elapsed < best {
                best = elapsed;
            }
        }

        println!("finished baseline of {} ms!", best.as_millis());
        Ok(best)
    }
}

impl Measurement for RelativeWallTime {
    type Intermediate = Instant;
    type Value = Duration;

    fn start(&self) -> Self::Intermediate {
        Instant::now()
    }

    fn end(&self, i: Self::Intermediate) -> Self::Value {
        let elap = i.elapsed();
        if elap < self.offset {
            Duration::new(0, 0)
        } else {
            elap - self.offset
        }
    }

    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value {
        *v1 + *v2
    }

    fn zero(&self) -> Self::Value {
        Duration::from_secs(0)
    }

    fn to_f64(&self, val: &Self::Value) -> f64 {
        val.as_nanos() as f64
    }

    fn formatter(&self) -> &dyn ValueFormatter {
        self.walltime.formatter()
    }
}
