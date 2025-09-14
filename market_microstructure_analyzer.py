#!/usr/bin/env python3
"""
Advanced Market Microstructure Analysis
Deep dive into order flow, price discovery, and trading patterns
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

class MarketMicrostructureAnalyzer:
    """Advanced market microstructure analysis for range bar data"""

    def __init__(self, json_path: str = None):
        """Initialize with range bar JSON data"""
        if json_path is None:
            json_path = "adversarial_test_output/BTCUSDT_rangebar_20250901_20250901_0.800pct.json"

        self.json_path = Path(json_path)
        self.data = None
        self.range_bars_df = None
        self.SCALE = 100_000_000  # 1e8 fixed-point scaling

        print(f"🔬 Initializing Market Microstructure Analyzer")
        print(f"📁 Data source: {self.json_path}")

    def load_and_prepare_data(self):
        """Load data and prepare microstructure metrics"""
        print("\n🔄 Loading and preparing microstructure data...")

        with open(self.json_path, 'r') as f:
            self.data = json.load(f)

        range_bars = self.data['range_bars']

        # Create comprehensive microstructure dataset
        processed_bars = []
        for i, bar in enumerate(range_bars):
            processed_bar = {
                'bar_index': i,
                'sequence_time': i,  # Sequential time for analysis
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

            # Advanced microstructure metrics
            processed_bar['price_change'] = processed_bar['close'] - processed_bar['open']
            processed_bar['price_change_pct'] = (processed_bar['price_change'] / processed_bar['open']) * 100
            processed_bar['is_bullish'] = processed_bar['close'] > processed_bar['open']
            processed_bar['direction'] = 1 if processed_bar['is_bullish'] else -1

            # Order flow metrics
            processed_bar['buy_ratio'] = processed_bar['buy_volume'] / processed_bar['volume'] if processed_bar['volume'] > 0 else 0.5
            processed_bar['sell_ratio'] = processed_bar['sell_volume'] / processed_bar['volume'] if processed_bar['volume'] > 0 else 0.5
            processed_bar['order_flow_imbalance'] = processed_bar['buy_ratio'] - processed_bar['sell_ratio']
            processed_bar['buy_trade_ratio'] = processed_bar['buy_trade_count'] / processed_bar['trade_count'] if processed_bar['trade_count'] > 0 else 0.5

            # Price discovery metrics
            processed_bar['body_size'] = abs(processed_bar['close'] - processed_bar['open'])
            processed_bar['upper_wick'] = processed_bar['high'] - max(processed_bar['open'], processed_bar['close'])
            processed_bar['lower_wick'] = min(processed_bar['open'], processed_bar['close']) - processed_bar['low']
            processed_bar['total_range'] = processed_bar['high'] - processed_bar['low']
            processed_bar['body_ratio'] = processed_bar['body_size'] / processed_bar['total_range'] if processed_bar['total_range'] > 0 else 0

            # VWAP analysis
            processed_bar['vwap_deviation'] = ((processed_bar['vwap'] - processed_bar['close']) / processed_bar['close']) * 100
            processed_bar['vwap_signal'] = 'Above' if processed_bar['close'] > processed_bar['vwap'] else 'Below'

            # Trade intensity
            processed_bar['avg_trade_size'] = processed_bar['volume'] / processed_bar['trade_count'] if processed_bar['trade_count'] > 0 else 0
            processed_bar['buy_avg_trade_size'] = processed_bar['buy_volume'] / processed_bar['buy_trade_count'] if processed_bar['buy_trade_count'] > 0 else 0
            processed_bar['sell_avg_trade_size'] = processed_bar['sell_volume'] / processed_bar['sell_trade_count'] if processed_bar['sell_trade_count'] > 0 else 0

            # Market impact metrics
            processed_bar['volume_normalized_move'] = processed_bar['price_change_pct'] / (processed_bar['volume'] / 1000) if processed_bar['volume'] > 0 else 0
            processed_bar['trade_normalized_move'] = processed_bar['price_change_pct'] / (processed_bar['trade_count'] / 1000) if processed_bar['trade_count'] > 0 else 0

            processed_bars.append(processed_bar)

        self.range_bars_df = pd.DataFrame(processed_bars)

        print(f"✅ Prepared microstructure data for {len(self.range_bars_df)} bars")
        print(f"📊 Microstructure metrics: {len([col for col in self.range_bars_df.columns if 'ratio' in col or 'imbalance' in col])} order flow indicators")

        return self

    def analyze_order_flow_patterns(self):
        """Analyze order flow patterns and correlations"""
        print("\n📈 Analyzing order flow patterns...")

        df = self.range_bars_df

        # Order flow correlation analysis
        order_flow_corr = df['order_flow_imbalance'].corr(df['price_change_pct'])
        buy_volume_corr = df['buy_ratio'].corr(df['direction'])
        trade_size_corr = df['avg_trade_size'].corr(df['body_size'])

        print(f"🎯 Order Flow Insights:")
        print(f"   💹 Flow-Price Correlation: r={order_flow_corr:.3f}")
        print(f"   📊 Buy Volume-Direction: r={buy_volume_corr:.3f}")
        print(f"   📦 Trade Size-Range: r={trade_size_corr:.3f}")

        # Create order flow analysis visualization
        fig = make_subplots(
            rows=2, cols=2,
            subplot_titles=[
                'Order Flow vs Price Movement',
                'Buy Pressure Over Time',
                'Trade Size Distribution by Direction',
                'VWAP Signal Effectiveness'
            ],
            specs=[[{"secondary_y": True}, {"secondary_y": False}],
                   [{"secondary_y": False}, {"secondary_y": False}]]
        )

        # 1. Order flow vs price movement scatter
        fig.add_trace(
            go.Scatter(
                x=df['order_flow_imbalance'],
                y=df['price_change_pct'],
                mode='markers',
                marker=dict(
                    size=df['volume']/100,  # Size by volume
                    color=df['direction'],
                    colorscale='RdYlGn',
                    showscale=True,
                    colorbar=dict(title="Direction")
                ),
                text=[f"Bar {i}: {row['buy_ratio']:.1%} buy" for i, row in df.iterrows()],
                name='Order Flow'
            ),
            row=1, col=1
        )

        # 2. Buy pressure over time
        fig.add_trace(
            go.Scatter(
                x=df['bar_index'],
                y=df['buy_ratio'],
                mode='lines+markers',
                name='Buy Ratio',
                line=dict(color='green', width=2)
            ),
            row=1, col=2
        )

        # Add 50% reference line
        fig.add_hline(y=0.5, line_dash="dash", line_color="gray", row=1, col=2)

        # 3. Trade size by direction
        bullish_trades = df[df['is_bullish']]['avg_trade_size']
        bearish_trades = df[~df['is_bullish']]['avg_trade_size']

        fig.add_trace(
            go.Histogram(
                x=bullish_trades,
                name='Bullish Bars',
                opacity=0.7,
                marker_color='green'
            ),
            row=2, col=1
        )

        fig.add_trace(
            go.Histogram(
                x=bearish_trades,
                name='Bearish Bars',
                opacity=0.7,
                marker_color='red'
            ),
            row=2, col=1
        )

        # 4. VWAP signal analysis
        vwap_above = df[df['vwap_signal'] == 'Above']['price_change_pct'].mean()
        vwap_below = df[df['vwap_signal'] == 'Below']['price_change_pct'].mean()

        fig.add_trace(
            go.Bar(
                x=['Price Above VWAP', 'Price Below VWAP'],
                y=[vwap_above, vwap_below],
                marker_color=['green' if vwap_above > 0 else 'red',
                             'green' if vwap_below > 0 else 'red'],
                name='VWAP Signal'
            ),
            row=2, col=2
        )

        # Update layout
        fig.update_layout(
            title='Market Microstructure Analysis Dashboard',
            height=800,
            showlegend=True
        )

        # Update axis labels
        fig.update_xaxes(title_text="Order Flow Imbalance", row=1, col=1)
        fig.update_yaxes(title_text="Price Change (%)", row=1, col=1)
        fig.update_xaxes(title_text="Bar Index", row=1, col=2)
        fig.update_yaxes(title_text="Buy Ratio", row=1, col=2)
        fig.update_xaxes(title_text="Average Trade Size (BTC)", row=2, col=1)
        fig.update_yaxes(title_text="Frequency", row=2, col=1)
        fig.update_xaxes(title_text="VWAP Signal", row=2, col=2)
        fig.update_yaxes(title_text="Avg Price Change (%)", row=2, col=2)

        # Save interactive plot
        output_dir = Path('statistical_analysis')
        output_dir.mkdir(exist_ok=True)

        fig.write_html(output_dir / 'order_flow_analysis.html')
        print(f"✅ Interactive order flow analysis saved: {output_dir / 'order_flow_analysis.html'}")

        return self

    def create_correlation_heatmap(self):
        """Create comprehensive correlation heatmap"""
        print("\n🔥 Creating correlation heatmap...")

        df = self.range_bars_df

        # Select key microstructure variables for correlation analysis
        corr_vars = [
            'price_change_pct', 'buy_ratio', 'order_flow_imbalance', 'volume',
            'trade_count', 'avg_trade_size', 'body_ratio', 'vwap_deviation',
            'volume_normalized_move', 'trade_normalized_move'
        ]

        # Calculate correlation matrix
        corr_matrix = df[corr_vars].corr()

        # Create interactive heatmap
        fig = go.Figure(data=go.Heatmap(
            z=corr_matrix.values,
            x=corr_matrix.columns,
            y=corr_matrix.columns,
            colorscale='RdBu',
            zmid=0,
            text=corr_matrix.round(3).values,
            texttemplate="%{text}",
            textfont={"size": 10},
            hovertemplate='%{x} vs %{y}<br>Correlation: %{z:.3f}<extra></extra>'
        ))

        fig.update_layout(
            title='Market Microstructure Correlation Matrix',
            width=800,
            height=800,
            xaxis_tickangle=-45
        )

        output_dir = Path('statistical_analysis')
        fig.write_html(output_dir / 'correlation_heatmap.html')
        print(f"✅ Correlation heatmap saved: {output_dir / 'correlation_heatmap.html'}")

        # Print key correlations
        print(f"\n🔍 Key Correlations Discovered:")
        key_pairs = [
            ('price_change_pct', 'order_flow_imbalance'),
            ('buy_ratio', 'price_change_pct'),
            ('volume', 'trade_count'),
            ('avg_trade_size', 'body_ratio')
        ]

        for var1, var2 in key_pairs:
            corr_val = corr_matrix.loc[var1, var2]
            significance = "Strong" if abs(corr_val) > 0.5 else "Moderate" if abs(corr_val) > 0.3 else "Weak"
            print(f"   📊 {var1} ↔ {var2}: r={corr_val:.3f} ({significance})")

        return self

    def analyze_price_discovery_efficiency(self):
        """Analyze price discovery and market efficiency"""
        print("\n🎯 Analyzing price discovery efficiency...")

        df = self.range_bars_df

        # Price discovery metrics
        avg_body_ratio = df['body_ratio'].mean()
        vwap_accuracy = (df['vwap_deviation'].abs().mean())
        price_efficiency = 1 - (df['upper_wick'] + df['lower_wick']).mean() / df['total_range'].mean()

        print(f"💡 Price Discovery Insights:")
        print(f"   📏 Body Ratio (Efficiency): {avg_body_ratio:.1%}")
        print(f"   🎯 VWAP Deviation: ±{vwap_accuracy:.3f}%")
        print(f"   ⚡ Price Efficiency: {price_efficiency:.1%}")

        # Create price discovery visualization
        fig, axes = plt.subplots(2, 2, figsize=(15, 10))
        fig.suptitle('Price Discovery & Market Efficiency Analysis', fontsize=16, fontweight='bold')

        # 1. Body ratio distribution
        ax = axes[0, 0]
        sns.histplot(data=df, x='body_ratio', kde=True, ax=ax, alpha=0.7)
        ax.axvline(avg_body_ratio, color='red', linestyle='--', alpha=0.8, label=f'Mean: {avg_body_ratio:.1%}')
        ax.set_title('Price Discovery Efficiency (Body Ratio)')
        ax.set_xlabel('Body / Total Range')
        ax.legend()

        # 2. VWAP deviation analysis
        ax = axes[0, 1]
        colors = ['green' if x > 0 else 'red' for x in df['vwap_deviation']]
        ax.bar(df['bar_index'], df['vwap_deviation'], color=colors, alpha=0.7)
        ax.axhline(0, color='black', linestyle='-', alpha=0.5)
        ax.set_title('VWAP Deviation by Bar')
        ax.set_xlabel('Bar Index')
        ax.set_ylabel('VWAP Deviation (%)')

        # 3. Order flow vs price efficiency
        ax = axes[1, 0]
        scatter = ax.scatter(df['order_flow_imbalance'], df['body_ratio'],
                           c=df['volume'], s=50, alpha=0.7, cmap='viridis')
        ax.set_xlabel('Order Flow Imbalance')
        ax.set_ylabel('Body Ratio (Efficiency)')
        ax.set_title('Order Flow vs Price Efficiency')
        plt.colorbar(scatter, ax=ax, label='Volume (BTC)')

        # 4. Market impact analysis
        ax = axes[1, 1]
        ax.scatter(df['volume'], df['price_change_pct'],
                  c=['green' if x > 0 else 'red' for x in df['price_change_pct']],
                  s=50, alpha=0.7)
        ax.set_xlabel('Volume (BTC)')
        ax.set_ylabel('Price Change (%)')
        ax.set_title('Volume vs Price Impact')

        plt.tight_layout()

        output_dir = Path('statistical_analysis')
        plt.savefig(output_dir / 'price_discovery_analysis.png', dpi=150, bbox_inches='tight')
        print(f"✅ Price discovery analysis saved: {output_dir / 'price_discovery_analysis.png'}")

        return self

    def generate_microstructure_report(self):
        """Generate comprehensive microstructure insights report"""
        print("\n📝 Generating microstructure insights report...")

        df = self.range_bars_df

        # Calculate key insights
        insights = {
            'order_flow': {
                'correlation': df['order_flow_imbalance'].corr(df['price_change_pct']),
                'avg_buy_pressure': df['buy_ratio'].mean(),
                'buy_pressure_volatility': df['buy_ratio'].std(),
                'strongest_flow_bar': df.loc[df['order_flow_imbalance'].abs().idxmax()],
            },
            'price_discovery': {
                'avg_efficiency': df['body_ratio'].mean(),
                'vwap_accuracy': df['vwap_deviation'].abs().mean(),
                'best_discovery_bar': df.loc[df['body_ratio'].idxmax()],
            },
            'trade_patterns': {
                'avg_trade_size': df['avg_trade_size'].mean(),
                'trade_size_consistency': 1 - df['avg_trade_size'].std() / df['avg_trade_size'].mean(),
                'volume_concentration': df['volume'].std() / df['volume'].mean(),
            }
        }

        # Create detailed report
        report = []
        report.append("# Market Microstructure Analysis Report")
        report.append(f"**Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        report.append(f"**Analysis**: Advanced order flow and price discovery patterns")
        report.append("")

        report.append("## Executive Summary")
        report.append("### Order Flow Analysis")
        flow = insights['order_flow']
        report.append(f"- **Order flow predictiveness**: r={flow['correlation']:.3f}")
        report.append(f"- **Average buy pressure**: {flow['avg_buy_pressure']:.1%}")
        report.append(f"- **Flow volatility**: ±{flow['buy_pressure_volatility']:.1%}")
        report.append("")

        report.append("### Price Discovery Efficiency")
        discovery = insights['price_discovery']
        report.append(f"- **Discovery efficiency**: {discovery['avg_efficiency']:.1%}")
        report.append(f"- **VWAP tracking accuracy**: ±{discovery['vwap_accuracy']:.3f}%")
        report.append("")

        report.append("### Trading Pattern Analysis")
        patterns = insights['trade_patterns']
        report.append(f"- **Average trade size**: {patterns['avg_trade_size']:.3f} BTC")
        report.append(f"- **Trade size consistency**: {patterns['trade_size_consistency']:.1%}")
        report.append(f"- **Volume concentration**: {patterns['volume_concentration']:.2f}")
        report.append("")

        report.append("## Key Actionable Insights")

        # Generate trading insights based on correlations
        if flow['correlation'] > 0.5:
            report.append("- **Strong order flow signal**: Buy/sell imbalance predicts price direction")
        elif flow['correlation'] > 0.3:
            report.append("- **Moderate order flow signal**: Some predictive value in buy/sell flows")
        else:
            report.append("- **Weak order flow signal**: Price movements largely independent of flows")

        if discovery['avg_efficiency'] > 0.7:
            report.append("- **Efficient price discovery**: Most price action in bar bodies (low noise)")
        elif discovery['avg_efficiency'] > 0.5:
            report.append("- **Moderate price discovery**: Balanced between signal and noise")
        else:
            report.append("- **Inefficient price discovery**: High noise-to-signal ratio")

        if flow['avg_buy_pressure'] > 0.55:
            report.append("- **Bullish flow bias**: Consistent buying pressure above 55%")
        elif flow['avg_buy_pressure'] < 0.45:
            report.append("- **Bearish flow bias**: Consistent selling pressure above 55%")
        else:
            report.append("- **Balanced flows**: No systematic directional bias")

        # Save report
        output_dir = Path('statistical_analysis')
        with open(output_dir / 'microstructure_insights.md', 'w') as f:
            f.write('\n'.join(report))

        print(f"✅ Microstructure insights saved: {output_dir / 'microstructure_insights.md'}")

        return insights

    def run_complete_microstructure_analysis(self):
        """Run complete market microstructure analysis"""
        print("🔬 Starting Complete Market Microstructure Analysis")
        print("=" * 60)

        try:
            self.load_and_prepare_data()
            self.analyze_order_flow_patterns()
            self.create_correlation_heatmap()
            self.analyze_price_discovery_efficiency()
            insights = self.generate_microstructure_report()

            print("\n🎉 Complete Microstructure Analysis Finished!")
            print("📁 Results saved in: statistical_analysis/")
            print("   🌐 order_flow_analysis.html - Interactive order flow dashboard")
            print("   🔥 correlation_heatmap.html - Interactive correlation matrix")
            print("   📊 price_discovery_analysis.png - Price discovery efficiency")
            print("   📝 microstructure_insights.md - Comprehensive insights report")

            return True, insights

        except Exception as e:
            print(f"❌ Microstructure analysis failed: {e}")
            return False, None

def main():
    """Main execution"""
    analyzer = MarketMicrostructureAnalyzer()
    success, insights = analyzer.run_complete_microstructure_analysis()

    if success:
        print("\n💎 SUCCESS: Advanced microstructure analysis complete!")
        print("🎯 Actionable trading insights generated from order flow patterns")
    else:
        print("\n💥 FAILED: Microstructure analysis encountered errors")

if __name__ == '__main__':
    main()