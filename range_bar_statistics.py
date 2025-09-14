#!/usr/bin/env python3
"""
Range Bar Statistical Analysis & Visualization
Advanced statistical insights from real market microstructure data
"""

import json
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
import plotly.graph_objects as go
import plotly.express as px
from plotly.subplots import make_subplots
import scipy.stats as stats
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Tuple, Any
import warnings
warnings.filterwarnings('ignore')

# Set style for professional visualizations
plt.style.use('seaborn-v0_8')
sns.set_palette("husl")

class RangeBarStatisticalAnalyzer:
    """Advanced statistical analysis of range bar market microstructure data"""

    def __init__(self, json_path: str = None):
        """Initialize with range bar JSON data"""
        if json_path is None:
            json_path = "adversarial_test_output/BTCUSDT_rangebar_20250901_20250901_0.800pct.json"

        self.json_path = Path(json_path)
        self.data = None
        self.range_bars_df = None
        self.metadata = None
        self.statistics = {}

        # Fixed-point scaling factor from Rust
        self.SCALE = 100_000_000  # 1e8

        print(f"📊 Initializing Statistical Analyzer")
        print(f"📁 Data source: {self.json_path}")

    def load_data(self):
        """Load and parse range bar JSON data"""
        print("\n🔄 Loading range bar data...")

        with open(self.json_path, 'r') as f:
            self.data = json.load(f)

        # Extract range bars and convert fixed-point values
        range_bars = self.data['range_bars']

        # Convert to DataFrame with proper scaling
        processed_bars = []
        for bar in range_bars:
            processed_bar = {
                'bar_index': len(processed_bars),
                'open_time': bar['open_time'],
                'close_time': bar['close_time'],
                'open': bar['open'] / self.SCALE,
                'high': bar['high'] / self.SCALE,
                'low': bar['low'] / self.SCALE,
                'close': bar['close'] / self.SCALE,
                'volume': bar['volume'] / self.SCALE,
                'buy_volume': bar['buy_volume'] / self.SCALE,
                'sell_volume': bar['sell_volume'] / self.SCALE,
                'trade_count': bar['trade_count'],
                'buy_trade_count': bar['buy_trade_count'],
                'sell_trade_count': bar['sell_trade_count'],
                'vwap': bar['vwap'] / self.SCALE,
                'buy_turnover': bar['buy_turnover'] / (self.SCALE * self.SCALE),  # Price * Volume
                'sell_turnover': bar['sell_turnover'] / (self.SCALE * self.SCALE),
            }

            # Calculate derived metrics
            processed_bar['duration_ms'] = bar['close_time'] - bar['open_time']
            processed_bar['duration_minutes'] = processed_bar['duration_ms'] / (1000 * 60)
            processed_bar['price_change'] = processed_bar['close'] - processed_bar['open']
            processed_bar['price_change_pct'] = (processed_bar['price_change'] / processed_bar['open']) * 100
            processed_bar['is_bullish'] = processed_bar['close'] > processed_bar['open']
            processed_bar['body_size'] = abs(processed_bar['close'] - processed_bar['open'])
            processed_bar['upper_wick'] = processed_bar['high'] - max(processed_bar['open'], processed_bar['close'])
            processed_bar['lower_wick'] = min(processed_bar['open'], processed_bar['close']) - processed_bar['low']
            processed_bar['buy_ratio'] = processed_bar['buy_volume'] / processed_bar['volume'] if processed_bar['volume'] > 0 else 0
            processed_bar['sell_ratio'] = processed_bar['sell_volume'] / processed_bar['volume'] if processed_bar['volume'] > 0 else 0
            processed_bar['buy_trade_ratio'] = processed_bar['buy_trade_count'] / processed_bar['trade_count'] if processed_bar['trade_count'] > 0 else 0
            processed_bar['avg_trade_size'] = processed_bar['volume'] / processed_bar['trade_count'] if processed_bar['trade_count'] > 0 else 0
            processed_bar['trades_per_minute'] = processed_bar['trade_count'] / processed_bar['duration_minutes'] if processed_bar['duration_minutes'] > 0 else 0

            # Convert timestamps to readable format
            try:
                processed_bar['open_time_readable'] = datetime.fromtimestamp(bar['open_time'] / 1_000_000).strftime('%H:%M:%S')
                processed_bar['close_time_readable'] = datetime.fromtimestamp(bar['close_time'] / 1_000_000).strftime('%H:%M:%S')
            except (ValueError, OSError):
                processed_bar['open_time_readable'] = str(bar['open_time'])
                processed_bar['close_time_readable'] = str(bar['close_time'])

            processed_bars.append(processed_bar)

        self.range_bars_df = pd.DataFrame(processed_bars)
        self.metadata = self.data.get('metadata', {})

        print(f"✅ Loaded {len(self.range_bars_df)} range bars")
        print(f"📊 Timespan: {processed_bars[0]['open_time_readable']} - {processed_bars[-1]['close_time_readable']}")
        print(f"💰 Total Volume: {self.range_bars_df['volume'].sum():.1f} BTC")
        print(f"🔢 Total Trades: {self.range_bars_df['trade_count'].sum():,}")

        return self

    def calculate_basic_statistics(self):
        """Calculate comprehensive descriptive statistics"""
        print("\n📈 Calculating basic statistics...")

        df = self.range_bars_df

        # Price movement statistics
        price_stats = {
            'mean_change_pct': df['price_change_pct'].mean(),
            'std_change_pct': df['price_change_pct'].std(),
            'skewness': stats.skew(df['price_change_pct']),
            'kurtosis': stats.kurtosis(df['price_change_pct']),
            'bullish_ratio': df['is_bullish'].mean(),
            'median_body_size': df['body_size'].median(),
        }

        # Volume statistics
        volume_stats = {
            'mean_volume': df['volume'].mean(),
            'std_volume': df['volume'].std(),
            'cv_volume': df['volume'].std() / df['volume'].mean(),  # Coefficient of variation
            'median_volume': df['volume'].median(),
            'mean_buy_ratio': df['buy_ratio'].mean(),
            'std_buy_ratio': df['buy_ratio'].std(),
        }

        # Duration statistics
        duration_stats = {
            'mean_duration_min': df['duration_minutes'].mean(),
            'std_duration_min': df['duration_minutes'].std(),
            'median_duration_min': df['duration_minutes'].median(),
            'min_duration_min': df['duration_minutes'].min(),
            'max_duration_min': df['duration_minutes'].max(),
        }

        # Trade intensity statistics
        trade_stats = {
            'mean_trades_per_bar': df['trade_count'].mean(),
            'std_trades_per_bar': df['trade_count'].std(),
            'mean_avg_trade_size': df['avg_trade_size'].mean(),
            'mean_trades_per_minute': df['trades_per_minute'].mean(),
            'total_trades': df['trade_count'].sum(),
        }

        # Microstructure statistics
        microstructure_stats = {
            'mean_buy_trade_ratio': df['buy_trade_ratio'].mean(),
            'buy_pressure_consistency': 1 - df['buy_ratio'].std(),  # Lower std = more consistent
            'order_flow_correlation': df['buy_ratio'].corr(df['price_change_pct']),
            'volume_price_correlation': df['volume'].corr(df['body_size']),
        }

        self.statistics = {
            'price_movement': price_stats,
            'volume_analysis': volume_stats,
            'duration_analysis': duration_stats,
            'trade_intensity': trade_stats,
            'microstructure': microstructure_stats,
            'summary': {
                'total_bars': len(df),
                'analysis_period': f"{df.iloc[0]['open_time_readable']} - {df.iloc[-1]['close_time_readable']}",
                'dominant_direction': 'Bullish' if price_stats['bullish_ratio'] > 0.5 else 'Bearish',
                'market_regime': self._classify_market_regime(price_stats, volume_stats),
            }
        }

        # Print key insights
        print("\n🎯 Key Statistical Insights:")
        print(f"   📊 Market Bias: {self.statistics['summary']['dominant_direction']} ({price_stats['bullish_ratio']:.1%} bullish bars)")
        print(f"   📈 Avg Price Move: {price_stats['mean_change_pct']:.3f}% ± {price_stats['std_change_pct']:.3f}%")
        print(f"   💧 Avg Buy Pressure: {volume_stats['mean_buy_ratio']:.1%} of volume")
        print(f"   ⏱️  Avg Bar Duration: {duration_stats['mean_duration_min']:.1f} minutes")
        print(f"   🎯 Order Flow Signal: r={microstructure_stats['order_flow_correlation']:.3f}")

        return self

    def _classify_market_regime(self, price_stats: Dict, volume_stats: Dict) -> str:
        """Classify market regime based on statistical characteristics"""
        volatility = price_stats['std_change_pct']
        volume_consistency = 1 - volume_stats['cv_volume']

        if volatility > 0.5:
            return "High Volatility"
        elif volume_consistency > 0.7:
            return "Stable Volume"
        else:
            return "Mixed Conditions"

    def generate_distribution_analysis(self):
        """Generate comprehensive distribution analysis and visualizations"""
        print("\n📊 Generating distribution analysis...")

        df = self.range_bars_df

        # Create distribution analysis figure
        fig, axes = plt.subplots(2, 3, figsize=(18, 12))
        fig.suptitle('Range Bar Statistical Distributions', fontsize=16, fontweight='bold')

        # 1. Price Change Distribution
        ax = axes[0, 0]
        sns.histplot(data=df, x='price_change_pct', kde=True, ax=ax, alpha=0.7)
        ax.axvline(df['price_change_pct'].mean(), color='red', linestyle='--', alpha=0.8, label='Mean')
        ax.axvline(0.8, color='green', linestyle='--', alpha=0.8, label='Target +0.8%')
        ax.axvline(-0.8, color='green', linestyle='--', alpha=0.8, label='Target -0.8%')
        ax.set_title('Price Change Distribution')
        ax.set_xlabel('Price Change (%)')
        ax.legend()

        # 2. Volume Distribution
        ax = axes[0, 1]
        sns.histplot(data=df, x='volume', kde=True, ax=ax, alpha=0.7)
        ax.axvline(df['volume'].mean(), color='red', linestyle='--', alpha=0.8, label='Mean')
        ax.axvline(df['volume'].median(), color='orange', linestyle='--', alpha=0.8, label='Median')
        ax.set_title('Volume Distribution')
        ax.set_xlabel('Volume (BTC)')
        ax.legend()

        # 3. Duration Distribution
        ax = axes[0, 2]
        sns.histplot(data=df, x='duration_minutes', kde=True, ax=ax, alpha=0.7)
        ax.axvline(df['duration_minutes'].mean(), color='red', linestyle='--', alpha=0.8, label='Mean')
        ax.set_title('Bar Duration Distribution')
        ax.set_xlabel('Duration (minutes)')
        ax.legend()

        # 4. Buy Ratio Distribution
        ax = axes[1, 0]
        sns.histplot(data=df, x='buy_ratio', kde=True, ax=ax, alpha=0.7)
        ax.axvline(0.5, color='gray', linestyle='--', alpha=0.8, label='Balanced (50%)')
        ax.axvline(df['buy_ratio'].mean(), color='red', linestyle='--', alpha=0.8, label='Mean')
        ax.set_title('Buy Pressure Distribution')
        ax.set_xlabel('Buy Volume Ratio')
        ax.legend()

        # 5. Trade Intensity Distribution
        ax = axes[1, 1]
        sns.histplot(data=df, x='trades_per_minute', kde=True, ax=ax, alpha=0.7)
        ax.axvline(df['trades_per_minute'].mean(), color='red', linestyle='--', alpha=0.8, label='Mean')
        ax.set_title('Trade Intensity Distribution')
        ax.set_xlabel('Trades per Minute')
        ax.legend()

        # 6. VWAP vs Price Comparison
        ax = axes[1, 2]
        df['vwap_vs_close'] = ((df['vwap'] - df['close']) / df['close']) * 100
        sns.histplot(data=df, x='vwap_vs_close', kde=True, ax=ax, alpha=0.7)
        ax.axvline(0, color='gray', linestyle='--', alpha=0.8, label='Perfect VWAP')
        ax.set_title('VWAP vs Close Price Deviation')
        ax.set_xlabel('VWAP Deviation (%)')
        ax.legend()

        plt.tight_layout()

        # Save distribution analysis
        output_dir = Path('statistical_analysis')
        output_dir.mkdir(exist_ok=True)

        plt.savefig(output_dir / 'distribution_analysis.png', dpi=150, bbox_inches='tight')
        print(f"✅ Saved distribution analysis: {output_dir / 'distribution_analysis.png'}")

        # Statistical tests
        self._perform_distribution_tests()

        return self

    def _perform_distribution_tests(self):
        """Perform statistical tests on distributions"""
        df = self.range_bars_df

        print("\n🧪 Statistical Distribution Tests:")

        # Test for normality
        price_shapiro = stats.shapiro(df['price_change_pct'])
        volume_shapiro = stats.shapiro(df['volume'])

        print(f"   📊 Price Changes Normality: p={price_shapiro.pvalue:.4f} {'✅ Normal' if price_shapiro.pvalue > 0.05 else '❌ Non-normal'}")
        print(f"   📦 Volume Normality: p={volume_shapiro.pvalue:.4f} {'✅ Normal' if volume_shapiro.pvalue > 0.05 else '❌ Non-normal'}")

        # Test if buy ratio is significantly different from 50%
        buy_ratio_test = stats.ttest_1samp(df['buy_ratio'], 0.5)
        print(f"   ⚖️  Buy/Sell Balance Test: p={buy_ratio_test.pvalue:.4f} {'✅ Balanced' if buy_ratio_test.pvalue > 0.05 else '❌ Biased'}")

        # Correlation significance tests
        volume_price_corr = df['volume'].corr(df['body_size'])
        n = len(df)
        t_stat = volume_price_corr * np.sqrt((n-2)/(1-volume_price_corr**2))
        p_value = 2 * (1 - stats.t.cdf(abs(t_stat), n-2))
        print(f"   🔗 Volume-Price Correlation: r={volume_price_corr:.3f}, p={p_value:.4f}")

    def generate_summary_report(self):
        """Generate comprehensive summary report"""
        print("\n📋 Generating Statistical Summary Report...")

        output_dir = Path('statistical_analysis')
        output_dir.mkdir(exist_ok=True)

        # Create summary report
        report = []
        report.append("# Range Bar Statistical Analysis Report")
        report.append(f"**Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        report.append(f"**Data Source**: {self.json_path}")
        report.append("")

        # Summary statistics
        stats = self.statistics
        report.append("## Executive Summary")
        report.append(f"- **Analysis Period**: {stats['summary']['analysis_period']}")
        report.append(f"- **Total Range Bars**: {stats['summary']['total_bars']}")
        report.append(f"- **Market Bias**: {stats['summary']['dominant_direction']}")
        report.append(f"- **Market Regime**: {stats['summary']['market_regime']}")
        report.append("")

        # Key findings
        report.append("## Key Statistical Findings")
        report.append("### Price Movement Analysis")
        price = stats['price_movement']
        report.append(f"- Average price change: {price['mean_change_pct']:.3f}% ± {price['std_change_pct']:.3f}%")
        report.append(f"- Bullish bias: {price['bullish_ratio']:.1%} of bars")
        report.append(f"- Movement distribution skewness: {price['skewness']:.3f}")
        report.append("")

        report.append("### Volume & Microstructure")
        volume = stats['volume_analysis']
        micro = stats['microstructure']
        report.append(f"- Average buy pressure: {volume['mean_buy_ratio']:.1%}")
        report.append(f"- Buy pressure volatility: ±{volume['std_buy_ratio']:.1%}")
        report.append(f"- Order flow correlation with price: r={micro['order_flow_correlation']:.3f}")
        report.append("")

        report.append("### Trading Activity")
        trade = stats['trade_intensity']
        duration = stats['duration_analysis']
        report.append(f"- Average trades per bar: {trade['mean_trades_per_bar']:.0f}")
        report.append(f"- Average bar duration: {duration['mean_duration_min']:.1f} minutes")
        report.append(f"- Trade intensity: {trade['mean_trades_per_minute']:.0f} trades/minute")
        report.append("")

        # Save report
        with open(output_dir / 'statistical_report.md', 'w') as f:
            f.write('\n'.join(report))

        print(f"✅ Statistical report saved: {output_dir / 'statistical_report.md'}")

        return self

    def run_complete_analysis(self):
        """Run complete statistical analysis pipeline"""
        print("🚀 Starting Complete Statistical Analysis Pipeline")
        print("=" * 60)

        try:
            self.load_data()
            self.calculate_basic_statistics()
            self.generate_distribution_analysis()
            self.generate_summary_report()

            print("\n🎉 Complete Statistical Analysis Finished!")
            print("📁 Results saved in: statistical_analysis/")
            print("   📊 distribution_analysis.png - Visual distributions")
            print("   📋 statistical_report.md - Comprehensive report")

            return True

        except Exception as e:
            print(f"❌ Analysis failed: {e}")
            return False

def main():
    """Main execution"""
    analyzer = RangeBarStatisticalAnalyzer()
    success = analyzer.run_complete_analysis()

    if success:
        print("\n💎 SUCCESS: Statistical analysis complete!")
        print("🔍 Key insights available in generated reports")
    else:
        print("\n💥 FAILED: Statistical analysis encountered errors")

if __name__ == '__main__':
    main()