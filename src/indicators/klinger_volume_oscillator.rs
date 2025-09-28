#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::core::{Error, Method, MovingAverageConstructor, ValueType, OHLCV};
use crate::core::{IndicatorConfig, IndicatorInstance, IndicatorResult};
use crate::helpers::{sign, MA};
use crate::methods::Cross;

/// Klinger Volume Oscillator
///
/// ## Links
///
/// * <https://en.wikipedia.org/wiki/Volume_analysis#Klinger_Volume_Oscillator>
/// * <https://www.investopedia.com/terms/k/klingeroscillator.asp>
///
/// # 2 values
///
/// * `main` value
///
/// Range in \(`-inf`; `+inf`\)
///
/// * `signal line` value
///
/// Range in \(`-inf`; `+inf`\)
///
/// # 2 signals
///
/// * When `main` value crosses `0.0` upwards, then returns full buy signal.
///   When `main` value crosses `0.0` downwards, then returns full sell signal.
///   Otherwise returns no signal.
///
/// * When `main` value crosses `signal line` value  upwards, then returns full buy signal.
///   When `main` value crosses `signal line` downwards, then returns full sell signal.
///   Otherwise returns no signal.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KlingerVolumeOscillator<M: MovingAverageConstructor = MA> {
	/// Fast moving average type.
	///
	/// Default is [`EMA(34)`](crate::methods::EMA).
	///
	/// Period range in \[`2`; [`PeriodType::MAX`](crate::core::PeriodType)\).
	pub ma1: M,
	/// Slow moving average type.
	///
	/// Default is [`EMA(55)`](crate::methods::EMA).
	///
	/// Period range in \[`2`; [`PeriodType::MAX`](crate::core::PeriodType)\).
	pub ma2: M,
	/// Signal line moving average type.
	///
	/// Default is [`EMA(13)`](crate::methods::EMA).
	///
	/// Period range in \[`2`; [`PeriodType::MAX`](crate::core::PeriodType)\).
	pub signal: M,
}

impl<M: MovingAverageConstructor> IndicatorConfig for KlingerVolumeOscillator<M> {
	type Instance = KlingerVolumeOscillatorInstance<M>;

	const NAME: &'static str = "KlingerVolumeOscillator";

	fn init<T: OHLCV>(self, candle: &T) -> Result<Self::Instance, Error> {
		if !self.validate() {
			return Err(Error::WrongConfig);
		}

		let cfg = self;
		Ok(Self::Instance {
			ma1: cfg.ma1.init(0.)?,
			ma2: cfg.ma2.init(0.)?,
			ma3: cfg.signal.init(0.)?,
			cross1: Cross::default(),
			cross2: Cross::default(),
			last_tp: candle.tp(),
			cfg,
		})
	}

	fn validate(&self) -> bool {
		self.ma1.is_similar_to(&self.ma2)
			&& self.ma1.ma_period() > 1
			&& self.signal.ma_period() > 1
			&& self.ma1.ma_period() < self.ma2.ma_period()
	}

	fn set(&mut self, name: &str, value: String) -> Result<(), Error> {
		match name {
			"ma1" => match value.parse() {
				Err(_) => return Err(Error::ParameterParse(name.to_string(), value.to_string())),
				Ok(value) => self.ma1 = value,
			},
			"ma2" => match value.parse() {
				Err(_) => return Err(Error::ParameterParse(name.to_string(), value.to_string())),
				Ok(value) => self.ma2 = value,
			},
			"signal" => match value.parse() {
				Err(_) => return Err(Error::ParameterParse(name.to_string(), value.to_string())),
				Ok(value) => self.signal = value,
			},

			_ => {
				return Err(Error::ParameterParse(name.to_string(), value));
			}
		};

		Ok(())
	}

	fn size(&self) -> (u8, u8) {
		(2, 2)
	}
}

impl Default for KlingerVolumeOscillator {
	fn default() -> Self {
		Self {
			ma1: MA::EMA(34),
			ma2: MA::EMA(55),
			signal: MA::EMA(13),
		}
	}
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KlingerVolumeOscillatorInstance<M: MovingAverageConstructor = MA> {
	cfg: KlingerVolumeOscillator<M>,

	ma1: M::Instance,
	ma2: M::Instance,
	ma3: M::Instance,
	cross1: Cross,
	cross2: Cross,
	last_tp: ValueType,
}

impl<M: MovingAverageConstructor> IndicatorInstance for KlingerVolumeOscillatorInstance<M> {
	type Config = KlingerVolumeOscillator<M>;

	fn config(&self) -> &Self::Config {
		&self.cfg
	}

	fn next<T: OHLCV>(&mut self, candle: &T) -> IndicatorResult {
		let tp = candle.tp();

		let d = tp - self.last_tp;
		self.last_tp = tp;

		// let vol = if d > 0. {
		// 	candle.volume()
		// } else if d < 0. {
		// 	-candle.volume()
		// } else {
		// 	0.
		// };

		let vol = sign(d) * candle.volume();

		let ma1: ValueType = self.ma1.next(&vol);
		let ma2: ValueType = self.ma2.next(&vol);
		let ko = ma1 - ma2;

		let ma3: ValueType = self.ma3.next(&ko);

		let s1 = self.cross1.next(&(ko, 0.));
		let s2 = self.cross2.next(&(ko, ma3));

		IndicatorResult::new(&[ko, ma3], &[s1, s2])
	}
}
