#!/usr/bin/env python3
"""
Enhanced Tradability Dashboard for 18 Tier-1 USDT Pairs
IMPROVED VERSION: Better readability, larger text, enhanced visibility
"""

import pandas as pd
import numpy as np
import plotly.graph_objects as go
import plotly.express as px
from plotly.subplots import make_subplots
import plotly.figure_factory as ff
from pathlib import Path
import json
from typing import Dict, List, Tuple, Any
from datetime import datetime

class EnhancedTradabilityDashboard:
    def __init__(self, csv_file_path: str):
        """Initialize enhanced dashboard with better visual presentation"""
        self.csv_path = Path(csv_file_path)
        self.df = pd.read_csv(self.csv_path)
        self.timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        
        # Define our 3 TRADE-FOCUSED metrics with precise weights
        self.metrics = {
            'trade_frequency': 'Trade Frequency (40%)',
            'consistency': 'Consistency (30%)', 
            'speed': 'Speed/Opportunity Rate (30%)'
        }
        
        self.rank_columns = [f'{metric}_rank' for metric in self.metrics.keys()]
        self.score_columns = [f'{metric}_score' for metric in self.metrics.keys()]
        
    def create_enhanced_dashboard(self) -> go.Figure:
        """Create enhanced dashboard optimized for readability and presentation"""
        
        # Create clean 1x2 layout with maximum space for heatmap
        fig = make_subplots(
            rows=1, cols=2,
            subplot_titles=[
                '🔥 RANKING HEATMAP MATRIX',
                '⭐ OVERALL TRADABILITY RANKING'
            ],
            specs=[
                [{"type": "xy"}, {"type": "xy"}]
            ],
            horizontal_spacing=0.08,
            column_widths=[0.75, 0.25]  # Give heatmap maximum space
        )
        
        # 1. ENHANCED RANKING HEATMAP MATRIX (Prominent position)
        self.add_enhanced_ranking_heatmap(fig, row=1, col=1)
        
        # 2. ENHANCED OVERALL RANKING (Top-right)
        self.add_enhanced_overall_ranking(fig, row=1, col=2)
        
        
        
        # Enhanced layout configuration
        fig.update_layout(
            title={
                'text': f'<b>🚀 PREMIUM USDT PAIRS TRADABILITY DASHBOARD</b><br>' +
                       f'<span style="font-size:16px;">18 Symbols × 5 Metrics Analysis</span><br>' +
                       f'<span style="font-size:14px; color:#666;">Rank 1 = Best | Green = High Performance</span>',
                'x': 0.5,
                'font': {'size': 24, 'color': '#2E86AB', 'family': 'Inconsolata'}
            },
            height=1200,  # Increased height
            width=1800,   # Increased width
            showlegend=False,
            template="plotly_white",
            font=dict(family="Inconsolata", size=14, color="#333333"),  # Inconsolata fixed-width font
            margin=dict(l=100, r=100, t=180, b=80)  # Increased top margin for title
        )
        
        return fig
    
    def add_enhanced_ranking_heatmap(self, fig: go.Figure, row: int, col: int):
        """Enhanced ranking heatmap with better visibility"""
        
        # Sort by overall rank for better presentation
        df_sorted = self.df.sort_values('overall_tradability_rank')
        symbols = df_sorted['symbol'].tolist()
        metric_names = list(self.metrics.values())
        
        # Create inverted rank matrix for color scale (lower rank = better = darker green)
        rank_matrix = []
        for _, row_data in df_sorted.iterrows():
            rank_row = [19 - row_data[rank_col] for rank_col in self.rank_columns]
            rank_matrix.append(rank_row)
        
        # Enhanced color scale
        colorscale = [
            [0.0, '#8B0000'],    # Dark red (worst)
            [0.2, '#DC143C'],    # Red
            [0.4, '#FF6347'],    # Tomato
            [0.5, '#FFD700'],    # Gold (middle)
            [0.7, '#9ACD32'],    # Yellow green  
            [0.85, '#32CD32'],   # Lime green
            [1.0, '#006400']     # Dark green (best)
        ]
        
        # Enhanced heatmap with better text
        heatmap = go.Heatmap(
            z=rank_matrix,
            x=metric_names,
            y=symbols,
            colorscale=colorscale,
            showscale=True,
            text=[[f"#{19-val}" for val in row] for row in rank_matrix],  # Show actual ranks
            texttemplate="%{text}",
            textfont={"size": 12, "color": "white", "family": "Inconsolata"},
            colorbar=dict(
                title=dict(
                    text="<b>PERFORMANCE<br>RANKING</b>",
                    font=dict(size=16, color="#333")
                ),
                tickmode="array",
                tickvals=[2, 6, 11, 16],
                ticktext=["#18 (Worst)", "#12", "#6", "#1 (Best)"],
                tickfont=dict(size=12),
                len=0.8,
                thickness=25,
                x=1.02
            ),
            hovertemplate=
            '<b>%{y}</b><br>' +
            'Metric: %{x}<br>' +
            'Rank: #%{customdata}<br>' +
            '<extra></extra>',
            customdata=[[19-val for val in row] for row in rank_matrix]
        )
        
        fig.add_trace(heatmap, row=row, col=col)
        
        # Enhanced axis formatting
        fig.update_xaxes(
            title_text="<b>PERFORMANCE METRICS</b>", 
            title_font_size=16,
            tickfont_size=14,
            row=row, col=col
        )
        fig.update_yaxes(
            title_text="<b>TRADING SYMBOLS</b>", 
            title_font_size=16,
            tickfont_size=14,
            row=row, col=col
        )
        
    def add_enhanced_overall_ranking(self, fig: go.Figure, row: int, col: int):
        """Enhanced overall ranking with better presentation"""
        
        df_sorted = self.df.sort_values('overall_tradability_rank')
        
        # Enhanced color mapping
        colors = []
        for rank in df_sorted['overall_tradability_rank']:
            if rank <= 3:
                colors.append('#006400')      # Dark green for top 3
            elif rank <= 6:
                colors.append('#32CD32')      # Green for top 6
            elif rank <= 12:
                colors.append('#FFD700')      # Gold for middle
            else:
                colors.append('#DC143C')      # Red for bottom
        
        bar_chart = go.Bar(
            x=df_sorted['overall_tradability_score'],
            y=df_sorted['symbol'],
            orientation='h',
            marker=dict(
                color=colors, 
                line=dict(color='#333333', width=1)
            ),
            text=[f"#{rank}" for rank in df_sorted['overall_tradability_rank']],
            textposition='outside',
            textfont=dict(size=13, color='#333', family='Inconsolata'),
            hovertemplate=
            '<b>%{y}</b><br>' +
            'Overall Score: %{x:.1f}<br>' +
            'Rank: %{text}<br>' +
            '<extra></extra>'
        )
        
        fig.add_trace(bar_chart, row=row, col=col)
        
        fig.update_xaxes(
            title_text="<b>Overall Tradability Score</b>", 
            title_font_size=14,
            tickfont_size=12,
            row=row, col=col
        )
        fig.update_yaxes(
            title_text="<b>Symbol</b>", 
            title_font_size=14,
            tickfont_size=12,
            row=row, col=col
        )
        
    def save_enhanced_dashboard(self, output_dir: str = "./output/tier1_analysis/tradability_analysis/"):
        """Save enhanced dashboard with high-quality output"""
        
        output_path = Path(output_dir)
        output_path.mkdir(parents=True, exist_ok=True)
        
        # Generate enhanced dashboard
        fig = self.create_enhanced_dashboard()
        
        # Save as HTML with enhanced configuration
        html_file = output_path / f"enhanced_tradability_dashboard_{self.timestamp}.html"
        fig.write_html(
            str(html_file), 
            include_plotlyjs='cdn',
            config={
                'displayModeBar': True,
                'displaylogo': False,
                'modeBarButtonsToRemove': ['pan2d', 'lasso2d']
            }
        )
        
        # Save high-quality PNG with explicit filename
        png_file = output_path / f"enhanced_dashboard_{self.timestamp}.png"
        try:
            fig.write_image(str(png_file), width=1800, height=1200, scale=2)  # High DPI
            print(f"   🖼️  High-Res PNG: {png_file}")
        except Exception as e:
            print(f"   ⚠️  PNG generation failed: {e}")
        
        # Save high-quality SVG for scalability
        svg_file = output_path / f"enhanced_dashboard_{self.timestamp}.svg"
        try:
            fig.write_image(str(svg_file), width=1800, height=1200)
            print(f"   📋 Scalable SVG: {svg_file}")
        except Exception as e:
            print(f"   ⚠️  SVG generation failed: {e}")
        
        print(f"📊 Enhanced Dashboard Saved:")
        print(f"   🌐 Interactive HTML: {html_file}")
        
        return str(html_file), str(png_file), str(svg_file)
    
    def print_enhanced_summary(self):
        """Print enhanced analysis summary"""
        
        print("🚀 ENHANCED TRADABILITY ANALYSIS SUMMARY")
        print("=" * 90)
        print(f"📊 Total Symbols Analyzed: {len(self.df)}")
        print(f"🎯 Metrics Evaluated: {len(self.metrics)}")
        print()
        
        # Top performers with enhanced formatting
        print("🏆 TOP 5 MOST TRADABLE SYMBOLS:")
        top_5 = self.df.nsmallest(5, 'overall_tradability_rank')
        for i, (_, row) in enumerate(top_5.iterrows(), 1):
            medal = ["🥇", "🥈", "🥉", "4️⃣", "5️⃣"][i-1]
            print(f"   {medal} {row['symbol']:8} | Score: {row['overall_tradability_score']:6.1f} | " +
                  f"Rankings → TF:{row['trade_frequency_rank']:2d} Con:{row['consistency_rank']:2d} Spd:{row['speed_rank']:2d}")
        print()
        
        # Category champions
        print("🎯 CATEGORY CHAMPIONS:")
        for metric, score_col in zip(self.metrics.values(), self.score_columns):
            best_idx = self.df[score_col].idxmax()
            best_symbol = self.df.loc[best_idx, 'symbol']
            best_score = self.df.loc[best_idx, score_col]
            print(f"   {metric:15}: {best_symbol:8} ({best_score:5.1f})")
        
        print()
        print("📋 Enhanced Dashboard Features:")
        print("   🔥 Larger, more readable heatmap matrix")
        print("   📊 Clean 2-chart layout with maximum space")
        print("   🎯 Improved color coding and legends")
        print("   📈 Higher resolution PNG and scalable SVG outputs")

def main():
    """Main execution function for enhanced dashboard"""
    
    # Find latest comprehensive CSV file
    analysis_dir = Path("./output/tier1_analysis/tradability_analysis/")
    csv_files = list(analysis_dir.glob("comprehensive_tradability_rankings_*.csv"))
    
    if not csv_files:
        print("❌ No tradability rankings CSV files found!")
        return
    
    # Use the latest file
    latest_csv = max(csv_files, key=lambda f: f.stat().st_mtime)
    print(f"📂 Using latest rankings: {latest_csv.name}")
    print()
    
    # Create enhanced dashboard
    dashboard = EnhancedTradabilityDashboard(str(latest_csv))
    
    # Print enhanced summary
    dashboard.print_enhanced_summary()
    
    # Generate and save enhanced dashboard
    html_file, png_file, svg_file = dashboard.save_enhanced_dashboard()
    
    print()
    print("🎉 Enhanced Interactive Dashboard Generated!")
    print(f"🌐 HTML: file://{Path(html_file).absolute()}")
    print(f"🖼️  PNG:  file://{Path(png_file).absolute()}")
    print(f"📋 SVG:  file://{Path(svg_file).absolute()}")
    
if __name__ == "__main__":
    main()