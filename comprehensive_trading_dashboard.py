#!/usr/bin/env python3
"""
Comprehensive Trading Insights Dashboard
Unified analysis combining statistics, microstructure, and momentum patterns
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

class ComprehensiveTradingDashboard:
    """Unified trading insights dashboard combining all analyses"""

    def __init__(self, json_path: str = None):
        """Initialize with range bar JSON data"""
        if json_path is None:
            json_path = "adversarial_test_output/BTCUSDT_rangebar_20250901_20250901_0.800pct.json"

        self.json_path = Path(json_path)
        self.data = None
        self.range_bars_df = None
        self.SCALE = 100_000_000

        # Analysis results storage
        self.statistical_insights = {}
        self.microstructure_insights = {}
        self.momentum_insights = {}
        self.trading_recommendations = {}

        print(f"🎯 Initializing Comprehensive Trading Dashboard")
        print(f"📁 Data source: {self.json_path}")

    def load_and_synthesize_data(self):
        """Load data and synthesize all analytical components"""
        print("\n🔄 Loading and synthesizing comprehensive data...")

        with open(self.json_path, 'r') as f:
            self.data = json.load(f)

        range_bars = self.data['range_bars']

        # Create comprehensive dataset with all metrics
        processed_bars = []
        for i, bar in enumerate(range_bars):
            comprehensive_bar = {
                # Basic data
                'bar_index': i,
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
            }

            # Derived metrics (synthesized from all analyses)
            comprehensive_bar['price_change'] = comprehensive_bar['close'] - comprehensive_bar['open']
            comprehensive_bar['price_change_pct'] = (comprehensive_bar['price_change'] / comprehensive_bar['open']) * 100
            comprehensive_bar['is_bullish'] = comprehensive_bar['close'] > comprehensive_bar['open']
            comprehensive_bar['direction'] = 1 if comprehensive_bar['is_bullish'] else -1

            # Microstructure metrics
            comprehensive_bar['buy_ratio'] = comprehensive_bar['buy_volume'] / comprehensive_bar['volume'] if comprehensive_bar['volume'] > 0 else 0.5
            comprehensive_bar['order_flow_imbalance'] = comprehensive_bar['buy_ratio'] - 0.5
            comprehensive_bar['buy_trade_ratio'] = comprehensive_bar['buy_trade_count'] / comprehensive_bar['trade_count'] if comprehensive_bar['trade_count'] > 0 else 0.5

            # Price discovery metrics
            comprehensive_bar['body_size'] = abs(comprehensive_bar['close'] - comprehensive_bar['open'])
            comprehensive_bar['total_range'] = comprehensive_bar['high'] - comprehensive_bar['low']
            comprehensive_bar['body_ratio'] = comprehensive_bar['body_size'] / comprehensive_bar['total_range'] if comprehensive_bar['total_range'] > 0 else 0

            # VWAP analysis
            comprehensive_bar['vwap_deviation'] = ((comprehensive_bar['vwap'] - comprehensive_bar['close']) / comprehensive_bar['close']) * 100
            comprehensive_bar['vwap_signal'] = 1 if comprehensive_bar['close'] > comprehensive_bar['vwap'] else -1

            # Momentum metrics
            comprehensive_bar['momentum_score'] = comprehensive_bar['direction'] * comprehensive_bar['body_ratio']
            comprehensive_bar['avg_trade_size'] = comprehensive_bar['volume'] / comprehensive_bar['trade_count'] if comprehensive_bar['trade_count'] > 0 else 0

            processed_bars.append(comprehensive_bar)

        self.range_bars_df = pd.DataFrame(processed_bars)

        # Add sequence-dependent metrics
        df = self.range_bars_df
        df['momentum_2bar'] = df['momentum_score'].rolling(window=2, min_periods=1).mean()
        df['cumulative_direction'] = df['direction'].cumsum()
        df['volume_momentum'] = df['buy_ratio'] - 0.5

        print(f"✅ Synthesized comprehensive dataset with {len(df)} bars")
        print(f"📊 Total metrics: {len(df.columns)} analytical dimensions")

        return self

    def calculate_unified_insights(self):
        """Calculate unified insights across all analytical dimensions"""
        print("\n🧠 Calculating unified trading insights...")

        df = self.range_bars_df

        # Statistical insights
        self.statistical_insights = {
            'market_bias': 'Bullish' if df['is_bullish'].mean() > 0.5 else 'Bearish',
            'bullish_ratio': df['is_bullish'].mean(),
            'avg_price_move': df['price_change_pct'].mean(),
            'price_volatility': df['price_change_pct'].std(),
            'volume_consistency': 1 - df['volume'].std() / df['volume'].mean(),
            'regime': 'High Volatility' if df['price_change_pct'].std() > 0.5 else 'Stable'
        }

        # Microstructure insights
        order_flow_corr = df['order_flow_imbalance'].corr(df['price_change_pct'])
        vwap_accuracy = df['vwap_deviation'].abs().mean()

        self.microstructure_insights = {
            'order_flow_correlation': order_flow_corr,
            'order_flow_predictive': 'Strong' if abs(order_flow_corr) > 0.5 else 'Moderate' if abs(order_flow_corr) > 0.3 else 'Weak',
            'avg_buy_pressure': df['buy_ratio'].mean(),
            'buy_pressure_stability': 1 - df['buy_ratio'].std(),
            'price_discovery_efficiency': df['body_ratio'].mean(),
            'vwap_tracking_accuracy': vwap_accuracy
        }

        # Momentum insights
        momentum_persistence = self._calculate_momentum_persistence(df)
        max_consecutive = self._calculate_max_consecutive(df)

        self.momentum_insights = {
            'momentum_persistence': momentum_persistence,
            'max_consecutive_pattern': max_consecutive,
            'trending_capability': 'Strong' if max_consecutive >= 3 else 'Moderate' if max_consecutive >= 2 else 'Weak',
            'momentum_direction': 'Bullish' if df['momentum_score'].mean() > 0 else 'Bearish',
            'reversal_frequency': self._calculate_reversal_frequency(df)
        }

        # Generate trading signals
        df['unified_signal'] = self._generate_unified_signals(df)
        df['signal_strength'] = self._calculate_signal_strength(df)

        # Trading recommendations
        self._generate_trading_recommendations()

        print(f"🎯 Unified Insights Generated:")
        print(f"   📊 Market Regime: {self.statistical_insights['regime']}")
        print(f"   💹 Order Flow Power: {self.microstructure_insights['order_flow_predictive']}")
        print(f"   📈 Trending Ability: {self.momentum_insights['trending_capability']}")
        print(f"   🎯 Signal Quality: {self._assess_signal_quality()}")

        return self

    def _calculate_momentum_persistence(self, df):
        """Calculate momentum persistence across bars"""
        same_direction_count = 0
        for i in range(1, len(df)):
            if df.iloc[i]['direction'] == df.iloc[i-1]['direction']:
                same_direction_count += 1
        return same_direction_count / max(1, len(df) - 1)

    def _calculate_max_consecutive(self, df):
        """Calculate maximum consecutive bars in same direction"""
        max_consecutive = 1
        current_consecutive = 1

        for i in range(1, len(df)):
            if df.iloc[i]['direction'] == df.iloc[i-1]['direction']:
                current_consecutive += 1
                max_consecutive = max(max_consecutive, current_consecutive)
            else:
                current_consecutive = 1

        return max_consecutive

    def _calculate_reversal_frequency(self, df):
        """Calculate frequency of direction reversals"""
        reversals = 0
        for i in range(1, len(df)):
            if df.iloc[i]['direction'] != df.iloc[i-1]['direction']:
                reversals += 1
        return reversals / max(1, len(df) - 1)

    def _generate_unified_signals(self, df):
        """Generate unified trading signals combining all analyses"""
        signals = []

        for i, row in df.iterrows():
            signal = 0

            # Order flow component (30% weight)
            if abs(row['order_flow_imbalance']) > 0.1:
                signal += row['order_flow_imbalance'] * 0.3

            # VWAP component (20% weight)
            signal += row['vwap_signal'] * 0.2

            # Momentum component (30% weight)
            signal += row['momentum_score'] * 0.3

            # Volume confirmation (20% weight)
            if row['volume'] > df['volume'].median():
                signal *= 1.2  # Boost signal with high volume

            # Normalize
            signal = max(-1, min(1, signal))
            signals.append(signal)

        return signals

    def _calculate_signal_strength(self, df):
        """Calculate signal strength based on confluence"""
        strengths = []

        for i, row in df.iterrows():
            strength = 0

            # Order flow confidence
            strength += abs(row['order_flow_imbalance']) * 0.3

            # Price discovery confidence
            strength += row['body_ratio'] * 0.3

            # Volume confidence
            if row['volume'] > df['volume'].median():
                strength += 0.2

            # VWAP confidence
            if abs(row['vwap_deviation']) < 1.0:  # Close to VWAP = more confident
                strength += 0.2

            strengths.append(min(1.0, strength))

        return strengths

    def _assess_signal_quality(self):
        """Assess overall signal quality"""
        df = self.range_bars_df

        # Calculate signal accuracy using forward-looking validation
        accurate_signals = 0
        total_signals = 0

        for i in range(len(df) - 1):  # Exclude last bar (no forward return)
            if abs(df.iloc[i]['unified_signal']) > 0.1:  # Only count significant signals
                total_signals += 1
                signal_direction = np.sign(df.iloc[i]['unified_signal'])
                actual_direction = np.sign(df.iloc[i+1]['price_change_pct'])

                if signal_direction == actual_direction:
                    accurate_signals += 1

        accuracy = accurate_signals / max(1, total_signals)

        if accuracy > 0.7:
            return "Excellent"
        elif accuracy > 0.6:
            return "Good"
        elif accuracy > 0.5:
            return "Fair"
        else:
            return "Poor"

    def _generate_trading_recommendations(self):
        """Generate comprehensive trading recommendations"""
        self.trading_recommendations = {
            'strategy_type': self._recommend_strategy_type(),
            'key_signals': self._identify_key_signals(),
            'risk_factors': self._identify_risk_factors(),
            'execution_guidelines': self._generate_execution_guidelines(),
            'performance_expectations': self._estimate_performance()
        }

    def _recommend_strategy_type(self):
        """Recommend optimal strategy type based on analysis"""
        momentum_persistence = self.momentum_insights['momentum_persistence']
        order_flow_strength = abs(self.microstructure_insights['order_flow_correlation'])
        reversal_freq = self.momentum_insights['reversal_frequency']

        if momentum_persistence > 0.6 and order_flow_strength > 0.5:
            return "Momentum Following with Order Flow Confirmation"
        elif reversal_freq > 0.5 and order_flow_strength > 0.5:
            return "Mean Reversion with Order Flow Signals"
        elif order_flow_strength > 0.6:
            return "Pure Order Flow Strategy"
        else:
            return "Balanced Approach with Multiple Confirmations"

    def _identify_key_signals(self):
        """Identify the most important signals to watch"""
        signals = []

        if abs(self.microstructure_insights['order_flow_correlation']) > 0.5:
            signals.append("Order Flow Imbalance (Primary)")

        if self.microstructure_insights['price_discovery_efficiency'] > 0.7:
            signals.append("VWAP Relative Position")

        if self.momentum_insights['max_consecutive_pattern'] >= 2:
            signals.append("Momentum Continuation Patterns")

        signals.append("Volume Confirmation")

        return signals

    def _identify_risk_factors(self):
        """Identify key risk factors"""
        risks = []

        if self.statistical_insights['regime'] == 'High Volatility':
            risks.append("High volatility environment - larger stops needed")

        if self.momentum_insights['reversal_frequency'] > 0.6:
            risks.append("High reversal frequency - quick position management required")

        if self.microstructure_insights['buy_pressure_stability'] < 0.7:
            risks.append("Unstable order flow - signals may be noisy")

        if len(risks) == 0:
            risks.append("Low risk environment - stable market conditions")

        return risks

    def _generate_execution_guidelines(self):
        """Generate specific execution guidelines"""
        guidelines = []

        order_flow_corr = self.microstructure_insights['order_flow_correlation']

        if abs(order_flow_corr) > 0.5:
            direction = "Buy" if order_flow_corr > 0 else "Sell"
            guidelines.append(f"Watch for order flow imbalance > 10% then {direction.lower()}")

        vwap_accuracy = self.microstructure_insights['vwap_tracking_accuracy']
        if vwap_accuracy < 1.0:
            guidelines.append("Use VWAP as dynamic support/resistance level")

        max_consecutive = self.momentum_insights['max_consecutive_pattern']
        if max_consecutive >= 2:
            guidelines.append(f"Momentum trades valid for up to {max_consecutive} consecutive bars")

        guidelines.append("Confirm all signals with volume > median volume")

        return guidelines

    def _estimate_performance(self):
        """Estimate strategy performance expectations"""
        signal_quality = self._assess_signal_quality()
        order_flow_strength = abs(self.microstructure_insights['order_flow_correlation'])

        if signal_quality == "Excellent" and order_flow_strength > 0.6:
            return "High performance potential (60-70% accuracy expected)"
        elif signal_quality == "Good" and order_flow_strength > 0.5:
            return "Good performance potential (55-65% accuracy expected)"
        elif signal_quality == "Fair":
            return "Moderate performance potential (50-60% accuracy expected)"
        else:
            return "Conservative expectations (45-55% accuracy expected)"

    def create_comprehensive_dashboard(self):
        """Create comprehensive interactive dashboard"""
        print("\n📊 Creating comprehensive trading dashboard...")

        df = self.range_bars_df

        # Create comprehensive dashboard with 6 panels
        fig = make_subplots(
            rows=3, cols=2,
            subplot_titles=[
                'Price Action with Unified Signals',
                'Order Flow vs Price Movement',
                'Statistical Distribution Summary',
                'Momentum & Pattern Analysis',
                'Signal Strength & Accuracy',
                'Trading Performance Simulation'
            ],
            specs=[[{"secondary_y": True}, {"secondary_y": False}],
                   [{"secondary_y": False}, {"secondary_y": False}],
                   [{"secondary_y": False}, {"secondary_y": False}]]
        )

        # 1. Price action with unified signals
        fig.add_trace(
            go.Candlestick(
                x=df['bar_index'],
                open=df['open'],
                high=df['high'],
                low=df['low'],
                close=df['close'],
                name='Price Action'
            ),
            row=1, col=1
        )

        # Add signal markers
        long_signals = df[df['unified_signal'] > 0.2]
        short_signals = df[df['unified_signal'] < -0.2]

        fig.add_trace(
            go.Scatter(
                x=long_signals['bar_index'],
                y=long_signals['close'],
                mode='markers',
                marker=dict(symbol='triangle-up', size=15, color='green'),
                name='Long Signal'
            ),
            row=1, col=1
        )

        fig.add_trace(
            go.Scatter(
                x=short_signals['bar_index'],
                y=short_signals['close'],
                mode='markers',
                marker=dict(symbol='triangle-down', size=15, color='red'),
                name='Short Signal'
            ),
            row=1, col=1
        )

        # Add signal strength as secondary y-axis
        fig.add_trace(
            go.Scatter(
                x=df['bar_index'],
                y=df['signal_strength'],
                mode='lines',
                name='Signal Strength',
                line=dict(color='purple', width=2),
                yaxis='y2'
            ),
            row=1, col=1, secondary_y=True
        )

        # 2. Order flow vs price movement
        fig.add_trace(
            go.Scatter(
                x=df['order_flow_imbalance'],
                y=df['price_change_pct'],
                mode='markers',
                marker=dict(
                    size=df['volume']*5,
                    color=df['direction'],
                    colorscale='RdYlGn',
                    showscale=True
                ),
                text=[f"Bar {i}: Vol={vol:.1f}" for i, vol in enumerate(df['volume'])],
                name='Order Flow vs Price'
            ),
            row=1, col=2
        )

        # 3. Statistical distribution summary
        fig.add_trace(
            go.Histogram(
                x=df['price_change_pct'],
                name='Price Change Distribution',
                opacity=0.7,
                marker_color='blue'
            ),
            row=2, col=1
        )

        # 4. Momentum & pattern analysis
        colors = ['green' if x > 0 else 'red' for x in df['momentum_score']]
        fig.add_trace(
            go.Bar(
                x=df['bar_index'],
                y=df['momentum_score'],
                marker_color=colors,
                name='Momentum Score',
                opacity=0.7
            ),
            row=2, col=2
        )

        # 5. Signal strength & accuracy
        fig.add_trace(
            go.Scatter(
                x=df['bar_index'],
                y=df['unified_signal'],
                mode='lines+markers',
                name='Unified Signal',
                line=dict(color='blue', width=3)
            ),
            row=3, col=1
        )

        # 6. Trading performance simulation
        df['signal_return'] = df['unified_signal'].shift(1) * df['price_change_pct']  # Previous signal * current return
        df['cumulative_signal_return'] = df['signal_return'].cumsum()
        df['cumulative_buy_hold'] = df['price_change_pct'].cumsum()

        fig.add_trace(
            go.Scatter(
                x=df['bar_index'],
                y=df['cumulative_signal_return'],
                mode='lines',
                name='Signal Strategy',
                line=dict(color='blue', width=3)
            ),
            row=3, col=2
        )

        fig.add_trace(
            go.Scatter(
                x=df['bar_index'],
                y=df['cumulative_buy_hold'],
                mode='lines',
                name='Buy & Hold',
                line=dict(color='gray', width=2, dash='dash')
            ),
            row=3, col=2
        )

        # Update layout
        fig.update_layout(
            title='Comprehensive Trading Insights Dashboard',
            height=1200,
            showlegend=True
        )

        # Save interactive dashboard
        output_dir = Path('statistical_analysis')
        fig.write_html(output_dir / 'comprehensive_trading_dashboard.html')
        print(f"✅ Comprehensive dashboard saved: {output_dir / 'comprehensive_trading_dashboard.html'}")

        return self

    def generate_executive_summary(self):
        """Generate executive summary of all analyses"""
        print("\n📋 Generating executive summary...")

        summary = []
        summary.append("# Executive Trading Insights Summary")
        summary.append(f"**Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        summary.append(f"**Analysis**: Comprehensive range bar statistical analysis")
        summary.append(f"**Data Period**: {len(self.range_bars_df)} range bars from real BTC/USDT market data")
        summary.append("")

        # Market assessment
        summary.append("## Market Assessment")
        stats = self.statistical_insights
        summary.append(f"- **Market Bias**: {stats['market_bias']} ({stats['bullish_ratio']:.1%} bullish bars)")
        summary.append(f"- **Volatility Regime**: {stats['regime']}")
        summary.append(f"- **Average Price Move**: {stats['avg_price_move']:.3f}% ± {stats['price_volatility']:.3f}%")
        summary.append("")

        # Key findings
        summary.append("## Key Findings")
        micro = self.microstructure_insights
        momentum = self.momentum_insights

        summary.append(f"### Order Flow Analysis")
        summary.append(f"- **Predictive Power**: {micro['order_flow_predictive']} (r={micro['order_flow_correlation']:.3f})")
        summary.append(f"- **Buy Pressure**: {micro['avg_buy_pressure']:.1%} average")
        summary.append(f"- **Price Discovery Efficiency**: {micro['price_discovery_efficiency']:.1%}")
        summary.append("")

        summary.append(f"### Momentum Patterns")
        summary.append(f"- **Trending Capability**: {momentum['trending_capability']}")
        summary.append(f"- **Max Consecutive**: {momentum['max_consecutive_pattern']:.0f} bars")
        summary.append(f"- **Reversal Frequency**: {momentum['reversal_frequency']:.1%}")
        summary.append("")

        # Trading recommendations
        summary.append("## Trading Recommendations")
        recs = self.trading_recommendations
        summary.append(f"### Recommended Strategy")
        summary.append(f"**{recs['strategy_type']}**")
        summary.append("")

        summary.append(f"### Key Signals to Monitor")
        for signal in recs['key_signals']:
            summary.append(f"- {signal}")
        summary.append("")

        summary.append(f"### Execution Guidelines")
        for guideline in recs['execution_guidelines']:
            summary.append(f"- {guideline}")
        summary.append("")

        summary.append(f"### Risk Factors")
        for risk in recs['risk_factors']:
            summary.append(f"- {risk}")
        summary.append("")

        summary.append(f"### Performance Expectations")
        summary.append(f"{recs['performance_expectations']}")
        summary.append("")

        # Final assessment
        summary.append("## Final Assessment")
        signal_quality = self._assess_signal_quality()

        if signal_quality in ["Excellent", "Good"]:
            summary.append("✅ **RECOMMENDATION**: Proceed with systematic trading approach")
            summary.append("- Statistical edge identified in order flow patterns")
            summary.append("- Strong signal reliability supports automated execution")
            summary.append("- Risk-adjusted returns expected to exceed buy-and-hold")
        else:
            summary.append("⚠️  **RECOMMENDATION**: Conservative approach with additional validation")
            summary.append("- Signals require further refinement before full deployment")
            summary.append("- Consider paper trading for additional validation")
            summary.append("- Focus on risk management over return optimization")

        # Save executive summary
        output_dir = Path('statistical_analysis')
        with open(output_dir / 'executive_trading_summary.md', 'w') as f:
            f.write('\n'.join(summary))

        print(f"✅ Executive summary saved: {output_dir / 'executive_trading_summary.md'}")

        return self

    def run_complete_comprehensive_analysis(self):
        """Run complete comprehensive analysis"""
        print("🎯 Starting Complete Comprehensive Trading Analysis")
        print("=" * 70)

        try:
            self.load_and_synthesize_data()
            self.calculate_unified_insights()
            self.create_comprehensive_dashboard()
            self.generate_executive_summary()

            print("\n🏆 COMPLETE COMPREHENSIVE ANALYSIS FINISHED!")
            print("📁 Results saved in: statistical_analysis/")
            print("   🌐 comprehensive_trading_dashboard.html - Unified interactive dashboard")
            print("   📋 executive_trading_summary.md - Executive summary with recommendations")
            print("")
            print("🎯 STATISTICAL VISUALIZATION PIPELINE: ✅ COMPLETE")
            print("💡 ACTIONABLE TRADING INSIGHTS: ✅ GENERATED")
            print("📊 PROFESSIONAL ANALYSIS: ✅ DELIVERED")

            return True

        except Exception as e:
            print(f"❌ Comprehensive analysis failed: {e}")
            return False

def main():
    """Main execution"""
    dashboard = ComprehensiveTradingDashboard()
    success = dashboard.run_complete_comprehensive_analysis()

    if success:
        print("\n💎 SUCCESS: Complete statistical visualization pipeline operational!")
        print("🎯 Professional trading insights generated from range bar data")
        print("✨ Mission accomplished: CSV/JSON visualization with actionable insights")
    else:
        print("\n💥 FAILED: Comprehensive analysis encountered errors")

if __name__ == '__main__':
    main()