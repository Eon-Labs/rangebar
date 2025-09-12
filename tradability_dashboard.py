#!/usr/bin/env python3
"""
Interactive Tradability Dashboard for 18 Premium USDT Pairs
Multi-perspective visualization of 5 validated metrics with comprehensive matrix view
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

class TradabilityDashboard:
    def __init__(self, csv_file_path: str):
        """Initialize dashboard with latest tradability rankings CSV"""
        self.csv_path = Path(csv_file_path)
        self.df = pd.read_csv(self.csv_path)
        self.timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        
        # Define our 5 validated metrics
        self.metrics = {
            'volatility': 'Volatility Score',
            'liquidity': 'Liquidity Score', 
            'consistency': 'Consistency Score',
            'speed': 'Speed Score',
            'volume_intensity': 'Volume Intensity Score'
        }
        
        self.rank_columns = [f'{metric}_rank' for metric in self.metrics.keys()]
        self.score_columns = [f'{metric}_score' for metric in self.metrics.keys()]
        
    def create_comprehensive_dashboard(self) -> go.Figure:
        """Create comprehensive multi-view dashboard with 6 visualization types"""
        
        # Create subplot layout: 2 rows, 3 columns for different views
        fig = make_subplots(
            rows=3, cols=2,
            subplot_titles=[
                '🔥 Ranking Heatmap Matrix (18 Symbols × 5 Metrics)',
                '⭐ Overall Tradability Ranking',
                '📊 Metric Performance Radar Chart',
                '🏆 Top 5 vs Bottom 5 Comparison',
                '📈 Score Distribution by Metric',
                '🎯 Symbol Performance Summary'
            ],
            specs=[
                [{"type": "xy"}, {"type": "xy"}],
                [{"type": "polar"}, {"type": "xy"}],
                [{"type": "xy"}, {"type": "table"}]
            ],
            vertical_spacing=0.12,
            horizontal_spacing=0.08
        )
        
        # 1. RANKING HEATMAP MATRIX (Top-left)
        self.add_ranking_heatmap(fig, row=1, col=1)
        
        # 2. OVERALL RANKING BAR CHART (Top-right)
        self.add_overall_ranking_chart(fig, row=1, col=2)
        
        # 3. RADAR CHART - TOP PERFORMERS (Middle-left)
        self.add_radar_chart(fig, row=2, col=1)
        
        # 4. TOP vs BOTTOM COMPARISON (Middle-right)
        self.add_top_vs_bottom_comparison(fig, row=2, col=2)
        
        # 5. SCORE DISTRIBUTION (Bottom-left)
        self.add_score_distribution(fig, row=3, col=1)
        
        # 6. SUMMARY METRICS TABLE (Bottom-right)
        self.add_summary_table(fig, row=3, col=2)
        
        # Update layout
        fig.update_layout(
            title={
                'text': f'🚀 Premium USDT Pairs Tradability Dashboard - {self.timestamp}<br><sub>Interactive Analysis of 18 Symbols across 5 Validated Metrics | Rank 1 = Best Performance</sub>',
                'x': 0.5,
                'font': {'size': 20, 'color': '#2E86AB'}
            },
            height=1400,
            width=1600,
            showlegend=False,
            template="plotly_white",
            font=dict(family="Arial, sans-serif", size=10, color="#333333"),
            margin=dict(l=80, r=80, t=120, b=60)
        )
        
        return fig
    
    def add_ranking_heatmap(self, fig: go.Figure, row: int, col: int):
        """Add ranking heatmap matrix - core visualization"""
        
        # Prepare data for heatmap (18 symbols × 5 metrics)
        symbols = self.df['symbol'].tolist()
        metric_names = list(self.metrics.values())
        
        # Create rank matrix (lower rank = better, so invert for color scale)
        rank_matrix = []
        for _, row_data in self.df.iterrows():
            rank_row = [19 - row_data[rank_col] for rank_col in self.rank_columns]  # Invert for color
            rank_matrix.append(rank_row)
        
        # Custom color scale: Green (best) to Red (worst)
        colorscale = [
            [0.0, '#d73027'],    # Rank 18 (worst) - Red
            [0.2, '#fc8d59'],    # Rank 14-15 - Orange
            [0.4, '#fee08b'],    # Rank 10-13 - Yellow
            [0.6, '#d9ef8b'],    # Rank 6-9 - Light Green
            [0.8, '#91d4aa'],    # Rank 3-5 - Medium Green
            [1.0, '#2d8659']     # Rank 1-2 (best) - Dark Green
        ]
        
        # Create heatmap
        heatmap = go.Heatmap(
            z=rank_matrix,
            x=metric_names,
            y=symbols,
            colorscale=colorscale,
            showscale=True,
            colorbar=dict(
                title="Performance<br>(Green=Best,<br>Red=Worst)",
                tickmode="array",
                tickvals=[1, 6, 11, 16],
                ticktext=["Rank 18", "Rank 13", "Rank 6", "Rank 1"],
                len=0.7,
                x=1.15
            ),
            hovertemplate=
            '<b>%{y}</b><br>' +
            'Metric: %{x}<br>' +
            'Rank: %{customdata}<br>' +
            '<extra></extra>',
            customdata=[[19-val for val in row] for row in rank_matrix]
        )
        
        fig.add_trace(heatmap, row=row, col=col)
        
    def add_overall_ranking_chart(self, fig: go.Figure, row: int, col: int):
        """Add overall tradability ranking bar chart"""
        
        # Sort by overall rank
        df_sorted = self.df.sort_values('overall_tradability_rank')
        
        # Color scale based on ranking
        colors = ['#2d8659' if r <= 6 else '#91d4aa' if r <= 12 else '#fc8d59' 
                 for r in df_sorted['overall_tradability_rank']]
        
        bar_chart = go.Bar(
            x=df_sorted['overall_tradability_score'],
            y=df_sorted['symbol'],
            orientation='h',
            marker=dict(color=colors, line=dict(color='#333333', width=0.5)),
            text=df_sorted['overall_tradability_rank'],
            textposition='outside',
            hovertemplate=
            '<b>%{y}</b><br>' +
            'Overall Score: %{x:.1f}<br>' +
            'Rank: %{text}<br>' +
            '<extra></extra>'
        )
        
        fig.add_trace(bar_chart, row=row, col=col)
        
        fig.update_xaxes(title_text="Overall Tradability Score", row=row, col=col)
        fig.update_yaxes(title_text="Symbol", row=row, col=col)
        
    def add_radar_chart(self, fig: go.Figure, row: int, col: int):
        """Add radar chart for top 3 performers"""
        
        top_3 = self.df.nsmallest(3, 'overall_tradability_rank')
        
        colors = ['#2d8659', '#66c2a5', '#91d4aa']
        
        for i, (_, symbol_data) in enumerate(top_3.iterrows()):
            scores = [symbol_data[score_col] for score_col in self.score_columns]
            scores.append(scores[0])  # Complete the circle
            
            metric_labels = list(self.metrics.values()) + [list(self.metrics.values())[0]]
            
            radar = go.Scatterpolar(
                r=scores,
                theta=metric_labels,
                fill='toself',
                name=f"{symbol_data['symbol']} (Rank {symbol_data['overall_tradability_rank']})",
                line=dict(color=colors[i], width=2),
                fillcolor=colors[i],
                opacity=0.3
            )
            
            fig.add_trace(radar, row=row, col=col)
        
        fig.update_polars(
            radialaxis=dict(visible=True, range=[0, 100]),
            angularaxis=dict(tickfont=dict(size=8)),
            row=row, col=col
        )
        
    def add_top_vs_bottom_comparison(self, fig: go.Figure, row: int, col: int):
        """Add top 5 vs bottom 5 comparison"""
        
        top_5 = self.df.nsmallest(5, 'overall_tradability_rank')
        bottom_5 = self.df.nlargest(5, 'overall_tradability_rank')
        
        # Calculate average scores for each metric
        top_avg = [top_5[score_col].mean() for score_col in self.score_columns]
        bottom_avg = [bottom_5[score_col].mean() for score_col in self.score_columns]
        
        metric_names = list(self.metrics.values())
        
        # Top 5 average
        fig.add_trace(go.Bar(
            name='Top 5 Average',
            x=metric_names,
            y=top_avg,
            marker_color='#2d8659',
            opacity=0.8,
            hovertemplate='<b>Top 5 Avg</b><br>%{x}: %{y:.1f}<extra></extra>'
        ), row=row, col=col)
        
        # Bottom 5 average
        fig.add_trace(go.Bar(
            name='Bottom 5 Average',
            x=metric_names,
            y=bottom_avg,
            marker_color='#d73027',
            opacity=0.8,
            hovertemplate='<b>Bottom 5 Avg</b><br>%{x}: %{y:.1f}<extra></extra>'
        ), row=row, col=col)
        
        fig.update_xaxes(title_text="Metrics", tickangle=45, row=row, col=col)
        fig.update_yaxes(title_text="Average Score", row=row, col=col)
        
    def add_score_distribution(self, fig: go.Figure, row: int, col: int):
        """Add score distribution violin plots"""
        
        # Prepare data for violin plot
        all_scores = []
        all_metrics = []
        
        for metric, score_col in zip(self.metrics.values(), self.score_columns):
            scores = self.df[score_col].tolist()
            all_scores.extend(scores)
            all_metrics.extend([metric] * len(scores))
        
        violin = go.Violin(
            x=all_metrics,
            y=all_scores,
            box_visible=True,
            meanline_visible=True,
            fillcolor='lightblue',
            opacity=0.6,
            line_color='darkblue'
        )
        
        fig.add_trace(violin, row=row, col=col)
        
        fig.update_xaxes(title_text="Metrics", tickangle=45, row=row, col=col)
        fig.update_yaxes(title_text="Score Distribution", row=row, col=col)
        
    def add_summary_table(self, fig: go.Figure, row: int, col: int):
        """Add summary statistics table"""
        
        # Calculate summary statistics
        summary_data = []
        for metric, score_col in zip(self.metrics.values(), self.score_columns):
            best_symbol = self.df.loc[self.df[score_col].idxmax(), 'symbol']
            worst_symbol = self.df.loc[self.df[score_col].idxmin(), 'symbol']
            avg_score = self.df[score_col].mean()
            
            summary_data.append([
                metric,
                f"{best_symbol} ({self.df[score_col].max():.1f})",
                f"{worst_symbol} ({self.df[score_col].min():.1f})",
                f"{avg_score:.1f}"
            ])
        
        table = go.Table(
            header=dict(
                values=['<b>Metric</b>', '<b>Best (Score)</b>', '<b>Worst (Score)</b>', '<b>Average</b>'],
                fill_color='#2E86AB',
                font=dict(color='white', size=10),
                align='center'
            ),
            cells=dict(
                values=list(zip(*summary_data)),
                fill_color=[['#f0f0f0', 'white'] * len(summary_data)],
                font=dict(color='#333333', size=9),
                align=['left', 'center', 'center', 'center'],
                height=25
            )
        )
        
        fig.add_trace(table, row=row, col=col)
    
    def save_dashboard(self, output_dir: str = "./output/premium_analysis/tradability_analysis/"):
        """Save interactive HTML dashboard"""
        
        output_path = Path(output_dir)
        output_path.mkdir(parents=True, exist_ok=True)
        
        # Generate dashboard
        fig = self.create_comprehensive_dashboard()
        
        # Save as HTML
        html_file = output_path / f"tradability_dashboard_{self.timestamp}.html"
        fig.write_html(str(html_file), include_plotlyjs='cdn')
        
        # Save as static image
        img_file = output_path / f"tradability_dashboard_{self.timestamp}.png"
        try:
            fig.write_image(str(img_file), width=1600, height=1400, scale=2)
        except Exception as e:
            print(f"⚠️  Could not save PNG (install kaleido): {e}")
        
        print(f"📊 Dashboard saved:")
        print(f"   🌐 Interactive HTML: {html_file}")
        print(f"   🖼️  Static Image: {img_file}")
        
        return str(html_file)
    
    def print_analysis_summary(self):
        """Print comprehensive analysis summary"""
        
        print("🚀 TRADABILITY ANALYSIS SUMMARY")
        print("=" * 80)
        print(f"📊 Total Symbols Analyzed: {len(self.df)}")
        print(f"🎯 Metrics Evaluated: {len(self.metrics)}")
        print()
        
        # Top 5 performers
        top_5 = self.df.nsmallest(5, 'overall_tradability_rank')
        print("🏆 TOP 5 MOST TRADABLE SYMBOLS:")
        for i, (_, row) in enumerate(top_5.iterrows(), 1):
            print(f"   {i}. {row['symbol']:8} | Score: {row['overall_tradability_score']:5.1f} | "
                  f"Vol: {row['volatility_rank']:2d} | Liq: {row['liquidity_rank']:2d} | "
                  f"Con: {row['consistency_rank']:2d} | Spd: {row['speed_rank']:2d} | "
                  f"VI: {row['volume_intensity_rank']:2d}")
        print()
        
        # Bottom 5 performers
        bottom_5 = self.df.nlargest(5, 'overall_tradability_rank')
        print("⚠️  BOTTOM 5 LEAST TRADABLE SYMBOLS:")
        for i, (_, row) in enumerate(bottom_5.iterrows(), 1):
            rank = len(self.df) - i + 1
            print(f"  {rank:2d}. {row['symbol']:8} | Score: {row['overall_tradability_score']:5.1f} | "
                  f"Vol: {row['volatility_rank']:2d} | Liq: {row['liquidity_rank']:2d} | "
                  f"Con: {row['consistency_rank']:2d} | Spd: {row['speed_rank']:2d} | "
                  f"VI: {row['volume_intensity_rank']:2d}")
        print()
        
        # Metric leaders
        print("🎯 CATEGORY LEADERS:")
        for metric, score_col in zip(self.metrics.values(), self.score_columns):
            best_idx = self.df[score_col].idxmax()
            best_symbol = self.df.loc[best_idx, 'symbol']
            best_score = self.df.loc[best_idx, score_col]
            print(f"   {metric:20}: {best_symbol:8} ({best_score:.1f})")
        print()
        
        print("📋 Legend: Vol=Volatility, Liq=Liquidity, Con=Consistency, Spd=Speed, VI=Volume Intensity")
        print("📊 Rank 1 = Best Performance, Higher Rank = Lower Performance")

def main():
    """Main execution function"""
    
    # Find latest comprehensive CSV file
    analysis_dir = Path("./output/premium_analysis/tradability_analysis/")
    csv_files = list(analysis_dir.glob("comprehensive_tradability_rankings_*.csv"))
    
    if not csv_files:
        print("❌ No tradability rankings CSV files found!")
        return
    
    # Use the latest file
    latest_csv = max(csv_files, key=lambda f: f.stat().st_mtime)
    print(f"📂 Using latest rankings: {latest_csv.name}")
    print()
    
    # Create and save dashboard
    dashboard = TradabilityDashboard(str(latest_csv))
    
    # Print summary
    dashboard.print_analysis_summary()
    
    # Generate and save interactive dashboard
    html_file = dashboard.save_dashboard()
    
    print()
    print(f"🎉 Interactive dashboard generated successfully!")
    print(f"🌐 Open in browser: file://{Path(html_file).absolute()}")
    
if __name__ == "__main__":
    main()