#!/usr/bin/env python3
"""
Advanced Momentum & Pattern Analysis for Range Bars
Time series analysis, regime detection, and signal generation
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

class MomentumPatternAnalyzer:
    """Advanced momentum and pattern analysis for range bar sequences"""

    def __init__(self, json_path: str = None):
        """Initialize with range bar JSON data"""
        if json_path is None:
            json_path = "adversarial_test_output/BTCUSDT_rangebar_20250901_20250901_0.800pct.json"

        self.json_path = Path(json_path)
        self.data = None
        self.range_bars_df = None
        self.SCALE = 100_000_000  # 1e8 fixed-point scaling

        print(f"📊 Initializing Momentum & Pattern Analyzer")
        print(f"📁 Data source: {self.json_path}")

    def load_and_prepare_momentum_data(self):
        """Load data and prepare momentum/pattern metrics"""
        print("\n🔄 Loading and preparing momentum analysis data...")

        with open(self.json_path, 'r') as f:
            self.data = json.load(f)

        range_bars = self.data['range_bars']

        # Create comprehensive momentum dataset
        processed_bars = []
        for i, bar in enumerate(range_bars):
            processed_bar = {
                'bar_index': i,
                'sequence_time': i,
                'open': bar['open'] / self.SCALE,
                'high': bar['high'] / self.SCALE,
                'low': bar['low'] / self.SCALE,
                'close': bar['close'] / self.SCALE,
                'volume': bar['volume'] / self.SCALE,
                'buy_volume': bar['buy_volume'] / self.SCALE,
                'sell_volume': bar['sell_volume'] / self.SCALE,
                'trade_count': bar['trade_count'],
                'vwap': bar['vwap'] / self.SCALE,
            }

            # Basic momentum metrics
            processed_bar['price_change'] = processed_bar['close'] - processed_bar['open']
            processed_bar['price_change_pct'] = (processed_bar['price_change'] / processed_bar['open']) * 100
            processed_bar['is_bullish'] = processed_bar['close'] > processed_bar['open']
            processed_bar['direction'] = 1 if processed_bar['is_bullish'] else -1

            # Momentum indicators
            processed_bar['body_size'] = abs(processed_bar['close'] - processed_bar['open'])
            processed_bar['upper_wick'] = processed_bar['high'] - max(processed_bar['open'], processed_bar['close'])
            processed_bar['lower_wick'] = min(processed_bar['open'], processed_bar['close']) - processed_bar['low']
            processed_bar['total_range'] = processed_bar['high'] - processed_bar['low']

            # Volume momentum
            processed_bar['buy_ratio'] = processed_bar['buy_volume'] / processed_bar['volume'] if processed_bar['volume'] > 0 else 0.5
            processed_bar['volume_momentum'] = processed_bar['buy_ratio'] - 0.5  # Centered around 0

            # Price action strength
            processed_bar['strength'] = processed_bar['body_size'] / processed_bar['total_range'] if processed_bar['total_range'] > 0 else 0
            processed_bar['momentum_score'] = processed_bar['direction'] * processed_bar['strength']

            processed_bars.append(processed_bar)

        self.range_bars_df = pd.DataFrame(processed_bars)

        # Calculate sequence-dependent momentum indicators
        df = self.range_bars_df

        # Rolling momentum (2-bar and 3-bar windows)
        df['momentum_2bar'] = df['momentum_score'].rolling(window=2, min_periods=1).mean()
        df['momentum_3bar'] = df['momentum_score'].rolling(window=3, min_periods=1).mean()

        # Cumulative momentum
        df['cumulative_direction'] = df['direction'].cumsum()
        df['cumulative_momentum'] = df['momentum_score'].cumsum()

        # Regime detection
        df['high_vol_regime'] = df['total_range'] > df['total_range'].quantile(0.7)
        df['strong_momentum_regime'] = df['momentum_score'].abs() > df['momentum_score'].abs().quantile(0.7)

        # Pattern detection
        df['consecutive_direction'] = self._detect_consecutive_patterns(df['direction'])
        df['reversal_signal'] = self._detect_reversal_patterns(df)

        # Momentum persistence
        df['momentum_persistence'] = self._calculate_momentum_persistence(df)

        # Volume-price divergence
        df['vp_divergence'] = self._detect_volume_price_divergence(df)

        self.range_bars_df = df

        print(f"✅ Prepared momentum data for {len(df)} bars")
        print(f"📊 Momentum indicators: {len([col for col in df.columns if 'momentum' in col])} momentum metrics")
        print(f"🎯 Pattern signals: {len([col for col in df.columns if 'signal' in col or 'pattern' in col])} pattern indicators")

        return self

    def _detect_consecutive_patterns(self, direction_series):
        """Detect consecutive bullish/bearish patterns"""
        consecutive = []
        current_count = 0
        last_direction = None

        for direction in direction_series:
            if direction == last_direction:
                current_count += 1
            else:
                current_count = 1
                last_direction = direction

            consecutive.append(current_count * direction)

        return consecutive

    def _detect_reversal_patterns(self, df):
        """Detect potential reversal patterns"""
        reversal_signals = []

        for i in range(len(df)):
            signal = 0

            if i >= 2:
                # Pattern: Strong momentum followed by weakening
                recent_momentum = df.iloc[i-2:i+1]['momentum_score'].tolist()

                # Reversal pattern: momentum weakening after strong move
                if len(recent_momentum) >= 3:
                    if (abs(recent_momentum[0]) > 0.5 and
                        abs(recent_momentum[-1]) < abs(recent_momentum[0]) * 0.5):
                        signal = -recent_momentum[0] / abs(recent_momentum[0])  # Opposite direction

                # Volume divergence reversal
                if i >= 1:
                    prev_vol_momentum = df.iloc[i-1]['volume_momentum']
                    curr_vol_momentum = df.iloc[i]['volume_momentum']
                    prev_price_dir = df.iloc[i-1]['direction']
                    curr_price_dir = df.iloc[i]['direction']

                    # Volume going opposite to price (divergence)
                    if (prev_price_dir > 0 and curr_vol_momentum < prev_vol_momentum) or \
                       (prev_price_dir < 0 and curr_vol_momentum > prev_vol_momentum):
                        signal += -prev_price_dir * 0.5

            reversal_signals.append(signal)

        return reversal_signals

    def _calculate_momentum_persistence(self, df):
        """Calculate momentum persistence score"""
        persistence = []

        for i in range(len(df)):
            if i < 2:
                persistence.append(0)
                continue

            # Look at last 3 bars
            recent_momentum = df.iloc[max(0, i-2):i+1]['momentum_score'].tolist()

            # Calculate persistence as consistency of momentum direction
            directions = [1 if m > 0 else -1 if m < 0 else 0 for m in recent_momentum]
            if len(set(directions)) == 1 and directions[0] != 0:
                # All same direction
                avg_strength = np.mean([abs(m) for m in recent_momentum])
                persistence.append(directions[0] * avg_strength)
            else:
                # Mixed directions - low persistence
                persistence.append(0)

        return persistence

    def _detect_volume_price_divergence(self, df):
        """Detect volume-price divergence signals"""
        divergence = []

        for i in range(len(df)):
            if i < 1:
                divergence.append(0)
                continue

            prev_price_change = df.iloc[i-1]['price_change_pct']
            curr_price_change = df.iloc[i]['price_change_pct']
            prev_volume_momentum = df.iloc[i-1]['volume_momentum']
            curr_volume_momentum = df.iloc[i]['volume_momentum']

            # Divergence: price and volume momentum going in opposite directions
            price_direction_change = curr_price_change - prev_price_change
            volume_momentum_change = curr_volume_momentum - prev_volume_momentum

            # If price going up but volume momentum going down (or vice versa)
            if (price_direction_change > 0 and volume_momentum_change < 0) or \
               (price_direction_change < 0 and volume_momentum_change > 0):
                divergence.append(abs(price_direction_change) * abs(volume_momentum_change))
            else:
                divergence.append(0)

        return divergence

    def analyze_momentum_patterns(self):
        """Analyze momentum patterns and generate insights"""
        print("\n📈 Analyzing momentum patterns...")

        df = self.range_bars_df

        # Momentum statistics
        avg_momentum = df['momentum_score'].mean()
        momentum_volatility = df['momentum_score'].std()
        momentum_persistence_avg = df['momentum_persistence'].mean()

        # Pattern statistics
        max_consecutive = df['consecutive_direction'].abs().max()
        avg_consecutive = df['consecutive_direction'].abs().mean()
        reversal_frequency = (df['reversal_signal'].abs() > 0.1).sum() / len(df)

        # Regime analysis
        high_vol_pct = df['high_vol_regime'].mean()
        strong_momentum_pct = df['strong_momentum_regime'].mean()

        print(f"🎯 Momentum Pattern Insights:")
        print(f"   📊 Average Momentum: {avg_momentum:.3f} ± {momentum_volatility:.3f}")
        print(f"   🔄 Momentum Persistence: {momentum_persistence_avg:.3f}")
        print(f"   📏 Max Consecutive Bars: {max_consecutive:.0f}")
        print(f"   🔄 Reversal Frequency: {reversal_frequency:.1%}")
        print(f"   ⚡ High Volatility Regime: {high_vol_pct:.1%}")

        # Create momentum analysis visualization
        fig = make_subplots(
            rows=3, cols=2,
            subplot_titles=[
                'Momentum Score Progression',
                'Cumulative Momentum & Direction',
                'Consecutive Pattern Analysis',
                'Reversal Signal Detection',
                'Volume-Price Divergence',
                'Regime Detection'
            ],
            specs=[[{"secondary_y": False}, {"secondary_y": True}],
                   [{"secondary_y": False}, {"secondary_y": False}],
                   [{"secondary_y": False}, {"secondary_y": False}]]
        )

        # 1. Momentum score progression
        fig.add_trace(
            go.Scatter(
                x=df['bar_index'],
                y=df['momentum_score'],
                mode='lines+markers',
                name='Momentum Score',
                line=dict(color='blue', width=2),
                marker=dict(
                    color=df['direction'],
                    colorscale='RdYlGn',
                    size=8
                )
            ),
            row=1, col=1
        )

        # Add zero line
        fig.add_hline(y=0, line_dash="dash", line_color="gray", row=1, col=1)

        # 2. Cumulative momentum & direction
        fig.add_trace(
            go.Scatter(
                x=df['bar_index'],
                y=df['cumulative_momentum'],
                mode='lines',
                name='Cumulative Momentum',
                line=dict(color='purple', width=2)
            ),
            row=1, col=2
        )

        fig.add_trace(
            go.Scatter(
                x=df['bar_index'],
                y=df['cumulative_direction'],
                mode='lines',
                name='Cumulative Direction',
                line=dict(color='orange', width=2),
                yaxis='y2'
            ),
            row=1, col=2, secondary_y=True
        )

        # 3. Consecutive pattern analysis
        colors = ['green' if x > 0 else 'red' for x in df['consecutive_direction']]
        fig.add_trace(
            go.Bar(
                x=df['bar_index'],
                y=df['consecutive_direction'].abs(),
                marker_color=colors,
                name='Consecutive Count',
                opacity=0.7
            ),
            row=2, col=1
        )

        # 4. Reversal signal detection
        fig.add_trace(
            go.Scatter(
                x=df['bar_index'],
                y=df['reversal_signal'],
                mode='markers',
                name='Reversal Signal',
                marker=dict(
                    size=df['reversal_signal'].abs() * 20 + 5,
                    color=df['reversal_signal'],
                    colorscale='RdYlGn',
                    showscale=True
                )
            ),
            row=2, col=2
        )

        # 5. Volume-price divergence
        fig.add_trace(
            go.Scatter(
                x=df['bar_index'],
                y=df['vp_divergence'],
                mode='lines+markers',
                name='VP Divergence',
                line=dict(color='red', width=2),
                fill='tozeroy',
                fillcolor='rgba(255,0,0,0.3)'
            ),
            row=3, col=1
        )

        # 6. Regime detection
        regime_colors = []
        regime_labels = []
        for i, row in df.iterrows():
            if row['high_vol_regime'] and row['strong_momentum_regime']:
                regime_colors.append('red')
                regime_labels.append('High Vol + Strong Mom')
            elif row['high_vol_regime']:
                regime_colors.append('orange')
                regime_labels.append('High Volatility')
            elif row['strong_momentum_regime']:
                regime_colors.append('blue')
                regime_labels.append('Strong Momentum')
            else:
                regime_colors.append('gray')
                regime_labels.append('Normal')

        fig.add_trace(
            go.Scatter(
                x=df['bar_index'],
                y=[1] * len(df),
                mode='markers',
                marker=dict(
                    size=15,
                    color=regime_colors,
                ),
                text=regime_labels,
                name='Market Regime'
            ),
            row=3, col=2
        )

        # Update layout
        fig.update_layout(
            title='Advanced Momentum & Pattern Analysis Dashboard',
            height=1000,
            showlegend=True
        )

        # Save interactive plot
        output_dir = Path('statistical_analysis')
        output_dir.mkdir(exist_ok=True)

        fig.write_html(output_dir / 'momentum_pattern_analysis.html')
        print(f"✅ Interactive momentum analysis saved: {output_dir / 'momentum_pattern_analysis.html'}")

        return self

    def generate_trading_signals(self):
        """Generate comprehensive trading signals based on momentum patterns"""
        print("\n🎯 Generating trading signals...")

        df = self.range_bars_df

        # Signal generation logic
        signals = []
        signal_strength = []
        signal_type = []

        for i, row in df.iterrows():
            signal = 0
            strength = 0
            sig_type = 'HOLD'

            # Momentum continuation signals
            if row['momentum_persistence'] > 0.3:
                signal += row['momentum_persistence'] * 0.5
                strength += abs(row['momentum_persistence'])
                sig_type = 'MOMENTUM_LONG' if row['momentum_persistence'] > 0 else 'MOMENTUM_SHORT'

            # Reversal signals
            if abs(row['reversal_signal']) > 0.2:
                signal += row['reversal_signal']
                strength += abs(row['reversal_signal'])
                sig_type = 'REVERSAL_LONG' if row['reversal_signal'] > 0 else 'REVERSAL_SHORT'

            # Consecutive pattern signals
            if abs(row['consecutive_direction']) >= 3:
                # After 3+ consecutive, expect continuation with diminishing confidence
                continuation_prob = max(0.1, 0.7 - (abs(row['consecutive_direction']) - 3) * 0.1)
                signal += np.sign(row['consecutive_direction']) * continuation_prob * 0.3
                strength += continuation_prob * 0.3

            # Volume divergence signals (contrarian)
            if row['vp_divergence'] > 0.1:
                # High divergence suggests reversal
                prev_direction = df.iloc[max(0, i-1)]['direction'] if i > 0 else 0
                signal += -prev_direction * row['vp_divergence'] * 0.3
                strength += row['vp_divergence'] * 0.3

            # Regime-based signal adjustment
            if row['high_vol_regime']:
                strength *= 1.2  # Higher volatility = stronger signals needed
            if row['strong_momentum_regime']:
                strength *= 1.1  # Strong momentum regime

            # Normalize signal
            signal = max(-1, min(1, signal))
            strength = min(1, strength)

            signals.append(signal)
            signal_strength.append(strength)
            signal_type.append(sig_type)

        df['trading_signal'] = signals
        df['signal_strength'] = signal_strength
        df['signal_type'] = signal_type

        # Signal performance analysis
        signal_performance = self._analyze_signal_performance(df)

        print(f"📊 Signal Generation Results:")
        print(f"   🎯 Signals Generated: {len([s for s in signals if abs(s) > 0.1])}/{len(signals)}")
        print(f"   📈 Long Signals: {len([s for s in signals if s > 0.1])}")
        print(f"   📉 Short Signals: {len([s for s in signals if s < -0.1])}")
        print(f"   💪 Avg Signal Strength: {np.mean([s for s in signal_strength if s > 0]):.3f}")

        # Create signal visualization
        self._visualize_trading_signals(df)

        return signal_performance

    def _analyze_signal_performance(self, df):
        """Analyze trading signal performance"""
        performance = {}

        # Calculate forward returns for signal validation
        df['forward_return'] = df['price_change_pct'].shift(-1)  # Next bar's return

        # Signal accuracy
        long_signals = df[df['trading_signal'] > 0.1]
        short_signals = df[df['trading_signal'] < -0.1]

        if len(long_signals) > 0:
            long_accuracy = (long_signals['forward_return'] > 0).mean()
            long_avg_return = long_signals['forward_return'].mean()
        else:
            long_accuracy = 0
            long_avg_return = 0

        if len(short_signals) > 0:
            short_accuracy = (short_signals['forward_return'] < 0).mean()
            short_avg_return = short_signals['forward_return'].mean()
        else:
            short_accuracy = 0
            short_avg_return = 0

        performance = {
            'long_accuracy': long_accuracy,
            'short_accuracy': short_accuracy,
            'long_avg_return': long_avg_return,
            'short_avg_return': short_avg_return,
            'overall_accuracy': ((long_signals['forward_return'] > 0).sum() +
                               (short_signals['forward_return'] < 0).sum()) / max(1, len(long_signals) + len(short_signals))
        }

        return performance

    def _visualize_trading_signals(self, df):
        """Create trading signals visualization"""
        fig, axes = plt.subplots(3, 1, figsize=(15, 12))
        fig.suptitle('Trading Signal Analysis', fontsize=16, fontweight='bold')

        # 1. Price with signals
        ax = axes[0]
        ax.plot(df['bar_index'], df['close'], 'k-', linewidth=2, label='Close Price')

        # Mark signal points
        long_signals = df[df['trading_signal'] > 0.1]
        short_signals = df[df['trading_signal'] < -0.1]

        ax.scatter(long_signals['bar_index'], long_signals['close'],
                  c='green', s=100, marker='^', alpha=0.8, label='Long Signal')
        ax.scatter(short_signals['bar_index'], short_signals['close'],
                  c='red', s=100, marker='v', alpha=0.8, label='Short Signal')

        ax.set_title('Price Action with Trading Signals')
        ax.set_ylabel('Price (USDT)')
        ax.legend()
        ax.grid(True, alpha=0.3)

        # 2. Signal strength over time
        ax = axes[1]
        colors = ['green' if s > 0 else 'red' if s < 0 else 'gray' for s in df['trading_signal']]
        bars = ax.bar(df['bar_index'], df['trading_signal'], color=colors, alpha=0.7)

        # Add signal strength as bar height variation
        for i, (bar, strength) in enumerate(zip(bars, df['signal_strength'])):
            bar.set_alpha(0.3 + 0.7 * strength)  # Alpha based on strength

        ax.axhline(y=0, color='black', linestyle='-', alpha=0.5)
        ax.set_title('Trading Signal Strength')
        ax.set_ylabel('Signal Value')
        ax.grid(True, alpha=0.3)

        # 3. Cumulative signal performance
        ax = axes[2]
        df['signal_return'] = df['trading_signal'] * df['forward_return']  # Signal * next return
        df['cumulative_signal_return'] = df['signal_return'].cumsum()

        ax.plot(df['bar_index'], df['cumulative_signal_return'], 'b-', linewidth=2, label='Signal Strategy')
        ax.plot(df['bar_index'], df['price_change_pct'].cumsum(), 'g--', linewidth=2, label='Buy & Hold')

        ax.set_title('Cumulative Signal Performance')
        ax.set_xlabel('Bar Index')
        ax.set_ylabel('Cumulative Return (%)')
        ax.legend()
        ax.grid(True, alpha=0.3)

        plt.tight_layout()

        output_dir = Path('statistical_analysis')
        plt.savefig(output_dir / 'trading_signals_analysis.png', dpi=150, bbox_inches='tight')
        print(f"✅ Trading signals analysis saved: {output_dir / 'trading_signals_analysis.png'}")

    def generate_comprehensive_report(self, signal_performance):
        """Generate comprehensive momentum and pattern analysis report"""
        print("\n📝 Generating comprehensive momentum report...")

        df = self.range_bars_df

        # Generate insights
        report = []
        report.append("# Advanced Momentum & Pattern Analysis Report")
        report.append(f"**Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        report.append(f"**Analysis**: Time series momentum patterns and trading signal generation")
        report.append("")

        # Executive summary
        report.append("## Executive Summary")
        report.append("### Momentum Characteristics")
        avg_momentum = df['momentum_score'].mean()
        momentum_vol = df['momentum_score'].std()
        persistence = df['momentum_persistence'].mean()

        report.append(f"- **Average momentum**: {avg_momentum:.3f} ± {momentum_vol:.3f}")
        report.append(f"- **Momentum persistence**: {persistence:.3f}")
        report.append(f"- **Dominant direction**: {'Bullish' if avg_momentum > 0 else 'Bearish'}")
        report.append("")

        # Pattern analysis
        report.append("### Pattern Analysis")
        max_consecutive = df['consecutive_direction'].abs().max()
        reversal_freq = (df['reversal_signal'].abs() > 0.1).sum() / len(df)

        report.append(f"- **Max consecutive pattern**: {max_consecutive:.0f} bars")
        report.append(f"- **Reversal signal frequency**: {reversal_freq:.1%}")
        report.append("")

        # Signal performance
        report.append("### Trading Signal Performance")
        perf = signal_performance
        report.append(f"- **Long signal accuracy**: {perf['long_accuracy']:.1%}")
        report.append(f"- **Short signal accuracy**: {perf['short_accuracy']:.1%}")
        report.append(f"- **Overall signal accuracy**: {perf['overall_accuracy']:.1%}")
        report.append("")

        # Actionable insights
        report.append("## Key Trading Insights")

        if perf['overall_accuracy'] > 0.6:
            report.append("- **Strong signal reliability**: Trading signals show good predictive accuracy")
        elif perf['overall_accuracy'] > 0.5:
            report.append("- **Moderate signal reliability**: Some predictive value in trading signals")
        else:
            report.append("- **Weak signal reliability**: Signals need refinement or additional filters")

        if persistence > 0.2:
            report.append("- **Momentum persistence**: Trends tend to continue, favor momentum strategies")
        else:
            report.append("- **Mean reversion tendency**: Quick reversals common, favor contrarian strategies")

        if max_consecutive >= 3:
            report.append(f"- **Strong trending capability**: Market can sustain {max_consecutive:.0f}+ consecutive moves")

        if reversal_freq > 0.3:
            report.append("- **High reversal frequency**: Market exhibits frequent direction changes")

        # Save report
        output_dir = Path('statistical_analysis')
        with open(output_dir / 'momentum_pattern_report.md', 'w') as f:
            f.write('\n'.join(report))

        print(f"✅ Momentum pattern report saved: {output_dir / 'momentum_pattern_report.md'}")

        return self

    def run_complete_momentum_analysis(self):
        """Run complete momentum and pattern analysis"""
        print("📊 Starting Complete Momentum & Pattern Analysis")
        print("=" * 60)

        try:
            self.load_and_prepare_momentum_data()
            self.analyze_momentum_patterns()
            signal_performance = self.generate_trading_signals()
            self.generate_comprehensive_report(signal_performance)

            print("\n🎉 Complete Momentum Analysis Finished!")
            print("📁 Results saved in: statistical_analysis/")
            print("   🌐 momentum_pattern_analysis.html - Interactive momentum dashboard")
            print("   📊 trading_signals_analysis.png - Signal performance analysis")
            print("   📝 momentum_pattern_report.md - Comprehensive insights report")

            return True, signal_performance

        except Exception as e:
            print(f"❌ Momentum analysis failed: {e}")
            return False, None

def main():
    """Main execution"""
    analyzer = MomentumPatternAnalyzer()
    success, performance = analyzer.run_complete_momentum_analysis()

    if success:
        print("\n💎 SUCCESS: Advanced momentum & pattern analysis complete!")
        print("🎯 Trading signals generated with performance validation")
    else:
        print("\n💥 FAILED: Momentum analysis encountered errors")

if __name__ == '__main__':
    main()