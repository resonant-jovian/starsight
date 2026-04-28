//! Candlestick chart — starsight 0.3.0 showcase.
//!
//! Twelve-session synthetic OHLC sequence. Bodies are green for up days
//! (`close ≥ open`) and red for down days. Wicks extend from body top/bottom
//! to high/low. The default body width sits at ~70% of the per-session slot
//! so adjacent candles read distinctly.

use starsight::prelude::*;

fn main() -> Result<()> {
    let data = vec![
        Ohlc {
            timestamp: 0.0,
            open: 100.0,
            high: 110.0,
            low: 95.0,
            close: 105.0,
        },
        Ohlc {
            timestamp: 1.0,
            open: 105.0,
            high: 115.0,
            low: 100.0,
            close: 112.0,
        },
        Ohlc {
            timestamp: 2.0,
            open: 112.0,
            high: 118.0,
            low: 102.0,
            close: 104.0,
        },
        Ohlc {
            timestamp: 3.0,
            open: 104.0,
            high: 109.0,
            low: 95.0,
            close: 97.0,
        },
        Ohlc {
            timestamp: 4.0,
            open: 97.0,
            high: 105.0,
            low: 92.0,
            close: 102.0,
        },
        Ohlc {
            timestamp: 5.0,
            open: 102.0,
            high: 113.0,
            low: 100.0,
            close: 111.0,
        },
        Ohlc {
            timestamp: 6.0,
            open: 111.0,
            high: 120.0,
            low: 108.0,
            close: 118.0,
        },
        Ohlc {
            timestamp: 7.0,
            open: 118.0,
            high: 122.0,
            low: 110.0,
            close: 113.0,
        },
        Ohlc {
            timestamp: 8.0,
            open: 113.0,
            high: 116.0,
            low: 105.0,
            close: 108.0,
        },
        Ohlc {
            timestamp: 9.0,
            open: 108.0,
            high: 115.0,
            low: 105.0,
            close: 114.0,
        },
        Ohlc {
            timestamp: 10.0,
            open: 114.0,
            high: 121.0,
            low: 112.0,
            close: 119.0,
        },
        Ohlc {
            timestamp: 11.0,
            open: 119.0,
            high: 125.0,
            low: 115.0,
            close: 124.0,
        },
    ];

    Figure::new(900, 500)
        .title("Twelve-session OHLC")
        .x_label("session")
        .y_label("price")
        .add(CandlestickMark::new(data))
        .save("examples/scientific/candlestick.png")
}
