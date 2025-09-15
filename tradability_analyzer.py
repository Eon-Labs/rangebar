#!/usr/bin/env python3
"""
Tier-1 Symbol Tradability Analyzer
Analyzes 18 Tier-1 USDT pairs for spot market tradability based on range bar volatility characteristics
"""

import json
import glob
import pandas as pd
import numpy as np
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Tuple, Any

class TradabilityAnalyzer:
    def __init__(self, results_directory: str = "./output/tier1_analysis"):
        self.results_dir = Path(results_directory)
        self.execution_timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        self.tradability_metrics = {}
        
    def discover_completed_symbols(self) -> List[Dict[str, Any]]:
        """Discover all completed symbol analysis results"""
        print("🔍 Discovering completed symbol analyses...")
        
        # Look for individual result directories
        individual_dir = self.results_dir / "individual"
        if not individual_dir.exists():
            print(f"❌ Individual results directory not found: {individual_dir}")
            return []
        
        completed_symbols = []
        
        for symbol_dir in individual_dir.iterdir():
            if symbol_dir.is_dir():
                symbol = symbol_dir.name
                
                # Look for JSON files with comprehensive statistics
                json_files = list(symbol_dir.glob("*.json"))
                export_summaries = [f for f in json_files if "export_summary" in f.name]
                
                if export_summaries:
                    try:
                        with open(export_summaries[0], 'r') as f:
                            data = json.load(f)
                            
                        completed_symbols.append({
                            "symbol": symbol,
                            "data_file": str(export_summaries[0]),
                            "data": data,
                            "total_bars": data.get("total_bars", 0),
                            "total_trades": data.get("total_trades", 0),
                            "processing_time": data.get("processing_time_seconds", 0)
                        })
                        
                        print(f"✅ {symbol}: {data.get('total_bars', 0)} bars, {data.get('total_trades', 0)} trades")
                        
                    except Exception as e:
                        print(f"⚠️  {symbol}: Failed to load data - {e}")
        
        print(f"📊 Found {len(completed_symbols)} completed symbol analyses")
        return completed_symbols
    
    def extract_volatility_metrics(self, symbol_data: Dict[str, Any]) -> Dict[str, float]:
        """Extract key volatility metrics from symbol analysis"""
        
        try:
            metadata = symbol_data["data"].get("metadata", {})
            stats = metadata.get("statistics", {})
            
            # Market data statistics
            market_data = stats.get("market_data", {})
            price_stats = market_data.get("price_stats", {})
            volume_stats = market_data.get("volume_stats", {})
            
            # Range bar statistics  
            range_bars = stats.get("range_bars", {})
            basic_stats = range_bars.get("basic_stats", {})
            duration_analysis = range_bars.get("duration_analysis", {})
            volume_analysis = range_bars.get("volume_analysis", {})
            price_efficiency = range_bars.get("price_efficiency", {})
            
            # Extract key volatility indicators
            volatility_metrics = {
                # Basic market characteristics
                "price_std_dev": price_stats.get("std_dev", 0.0),
                "price_mean": price_stats.get("mean", 0.0),
                "coefficient_of_variation": price_stats.get("std_dev", 0.0) / max(price_stats.get("mean", 1.0), 1.0),
                
                # REMOVED: dollar volume characteristics (eliminated)
                
                # Range bar efficiency metrics
                "total_bars": basic_stats.get("total_bars", 0),
                "completion_rate": basic_stats.get("completion_rate", 0.0),
                "avg_trades_per_bar": basic_stats.get("avg_trades_per_bar", 0.0),
                "bars_per_hour": basic_stats.get("bars_per_hour", 0.0),
                
                # Duration characteristics (volatility proxy)
                "duration_mean_seconds": duration_analysis.get("duration_stats", {}).get("mean", 0.0) / 1000.0,
                "duration_std_dev": duration_analysis.get("duration_stats", {}).get("std_dev", 0.0) / 1000.0,
                "duration_skewness": duration_analysis.get("duration_stats", {}).get("skewness", 0.0),
                
                # CORE METRIC: Only trade frequency matters for small traders
                # avg_trades_per_bar = number of individual trades per range bar
                
                # REMOVED: volume per bar characteristics (eliminated dollar volume)
                
                # Price efficiency (tradability proxy)
                "range_utilization_ratio": price_efficiency.get("range_utilization", {}).get("actual_to_theoretical_ratio", 0.0),
                "range_efficiency_score": price_efficiency.get("range_utilization", {}).get("range_efficiency_score", 0.0),
                
                # Processing characteristics
                "total_trades": symbol_data.get("total_trades", 0),
                "processing_time_seconds": symbol_data.get("processing_time", 0.0),
                "throughput_trades_per_sec": symbol_data.get("total_trades", 0) / max(symbol_data.get("processing_time", 1.0), 1.0)
            }
            
            return volatility_metrics
            
        except Exception as e:
            print(f"⚠️  Error extracting metrics for {symbol_data['symbol']}: {e}")
            return {}
    
    def calculate_all_scores_and_rankings(self, symbols_metrics: Dict[str, Dict[str, float]]) -> Dict[str, Dict[str, float]]:
        """Calculate validated tradability scores and rankings - only 5 valid metrics"""
        
        if not symbols_metrics:
            return {}
        
        print("🧮 Calculating validated tradability scores (5 metrics only)...")
        
        # Create DataFrame for easier manipulation
        df = pd.DataFrame(symbols_metrics).T
        
        # Handle missing or zero values
        df = df.fillna(0)
        
        # Initialize results dictionary
        all_scores = {}
        
        for symbol in df.index:
            symbol_metrics = df.loc[symbol]
            
            # === VALIDATED 5 CORE METRICS ===
            
            # REMOVED: Volatility score - redundant with speed metric
            
            # 2. Trade Frequency Score - ONLY trade activity (NO dollar volume)
            # Higher trades per bar = more market participation
            trade_frequency_score = 0.0
            if df['avg_trades_per_bar'].max() > 0:
                trade_frequency_score = (symbol_metrics['avg_trades_per_bar'] / df['avg_trades_per_bar'].max()) * 100
            
            # 3. Consistency Score - Pattern stability
            # Lower duration variance = more consistent bar formation patterns
            consistency_score = 0.0
            if symbol_metrics['duration_mean_seconds'] > 0:
                duration_cv = symbol_metrics['duration_std_dev'] / symbol_metrics['duration_mean_seconds']
                max_duration_cv = df.apply(lambda row: row['duration_std_dev'] / max(row['duration_mean_seconds'], 1), axis=1).max()
                if max_duration_cv > 0:
                    consistency_score = (1 - (duration_cv / max_duration_cv)) * 100
            
            # 4. Speed Score - Bar formation rate
            # Faster bar completion = more active, responsive market
            speed_score = 0.0
            if df['bars_per_hour'].max() > 0:
                speed_score = (symbol_metrics['bars_per_hour'] / df['bars_per_hour'].max()) * 100
            
            # NO VOLUME INTENSITY - eliminated dollar volume completely
            
            # Overall Composite Score (focused on trading activity)
            overall_tradability = (
                trade_frequency_score * 0.40 +   # 40% weight on trade frequency
                consistency_score * 0.30 +       # 30% weight on consistency
                speed_score * 0.30               # 30% weight on speed (bars per hour)
                # REMOVED: volatility (redundant with speed)
                # REMOVED: volume intensity (eliminated dollar volume)
            )
            
            all_scores[symbol] = {
                # Simplified 3 core scores - TRADE FREQUENCY FOCUSED
                "trade_frequency_score": trade_frequency_score,
                "consistency_score": consistency_score,
                "speed_score": speed_score,
                # REMOVED: volatility_score (redundant with speed)
                # REMOVED: volume_intensity_score (eliminated dollar volume)
                
                # Overall composite
                "overall_tradability": overall_tradability,
                
                # Raw metrics for reference
                "bars_per_hour": symbol_metrics['bars_per_hour'],
                "avg_trades_per_bar": symbol_metrics['avg_trades_per_bar'],
                "duration_mean_seconds": symbol_metrics['duration_mean_seconds'],
                # REMOVED: volume_bar_mean (eliminated dollar volume)
                # REMOVED: coefficient_of_variation (eliminated volatility)
                "total_bars": symbol_metrics['total_bars'],
                "total_trades": symbol_metrics['total_trades']
            }
        
        # === CALCULATE INDIVIDUAL RANKINGS (1 = best, higher = worse) ===
        ranking_metrics = [
            "trade_frequency_score", "consistency_score", "speed_score", "overall_tradability"
        ]
        
        for metric in ranking_metrics:
            # Sort by score (descending) and assign ranks (1 = highest score = best)
            ranked_symbols = sorted(all_scores.items(), key=lambda x: x[1][metric], reverse=True)
            for rank, (symbol, scores) in enumerate(ranked_symbols, 1):
                rank_key = f"{metric.replace('_score', '')}_rank"
                all_scores[symbol][rank_key] = rank
        
        return all_scores
    
    def generate_tradability_report(self, symbols_metrics: Dict[str, Dict[str, float]], 
                                  tradability_scores: Dict[str, Dict[str, float]]) -> Dict[str, Any]:
        """Generate comprehensive tradability analysis report"""
        
        print("📊 Generating tradability analysis report...")
        
        # Overall rankings
        overall_ranking = sorted(tradability_scores.items(), 
                               key=lambda x: x[1]["overall_tradability"], 
                               reverse=True)
        
        # Category leaders (3 TRADE-FOCUSED metrics only)
        trade_frequency_leader = max(tradability_scores.items(), key=lambda x: x[1]["trade_frequency_score"])
        consistency_leader = max(tradability_scores.items(), key=lambda x: x[1]["consistency_score"])
        speed_leader = max(tradability_scores.items(), key=lambda x: x[1]["speed_score"])
        
        # Statistical summary
        overall_scores = [scores["overall_tradability"] for scores in tradability_scores.values()]
        
        report = {
            "analysis_metadata": {
                "execution_timestamp": self.execution_timestamp,
                "analysis_type": "spot_market_tradability_analysis",
                "symbols_analyzed": len(tradability_scores),
                "analysis_period": "6_months",
                "threshold_pct": "0.8%",
                "data_source": "binance_spot_aggtrades"
            },
            
            "executive_summary": {
                "most_tradable_symbol": overall_ranking[0][0],
                "least_tradable_symbol": overall_ranking[-1][0],
                "average_tradability_score": np.mean(overall_scores),
                "tradability_score_std_dev": np.std(overall_scores),
                "top_tier_symbols": [symbol for symbol, _ in overall_ranking[:6]],  # Top 1/3
                "bottom_tier_symbols": [symbol for symbol, _ in overall_ranking[-6:]]  # Bottom 1/3
            },
            
            "category_leaders": {
                "highest_trade_frequency": trade_frequency_leader[0], 
                "highest_consistency": consistency_leader[0],
                "highest_speed": speed_leader[0]
            },
            
            "detailed_rankings": {
                "overall_ranking": [
                    {
                        "rank": i + 1,
                        "symbol": symbol,
                        "overall_score": round(scores["overall_tradability"], 2),
                        "trade_frequency_score": round(scores["trade_frequency_score"], 2),
                        "consistency_score": round(scores["consistency_score"], 2),
                        "speed_score": round(scores["speed_score"], 2)
                    }
                    for i, (symbol, scores) in enumerate(overall_ranking)
                ]
            },
            
            "raw_metrics": symbols_metrics,
            "calculated_scores": tradability_scores,
            
            "claude_code_discovery": {
                "workspace_path": "/Users/terryli/eon/rangebar",
                "analysis_ready": True,
                "machine_readable": True,
                "tags": ["tradability_analysis", "spot_volatility", "18_tier1_symbols", "6_months"],
                "discovery_file": f"tradability_analysis_{self.execution_timestamp}_discovery.json"
            }
        }
        
        return report
    
    def save_analysis_results(self, report: Dict[str, Any]):
        """Save tradability analysis results with comprehensive CSV outputs"""
        
        # Ensure output directory exists
        output_dir = self.results_dir / "tradability_analysis"
        output_dir.mkdir(exist_ok=True)
        
        # Save comprehensive JSON report
        report_file = output_dir / f"tier1_tradability_analysis_{self.execution_timestamp}.json"
        with open(report_file, 'w') as f:
            json.dump(report, f, indent=2)
        
        # === CREATE COMPREHENSIVE CSV RANKINGS ===
        
        # Extract data for CSV creation
        calculated_scores = report["calculated_scores"]
        
        # Create master rankings DataFrame
        rankings_data = []
        for symbol, scores in calculated_scores.items():
            ranking_row = {
                "symbol": symbol,
                
                # Validated 3 core rankings (1 = best)
                "trade_frequency_rank": scores.get("trade_frequency_rank", 0),
                "consistency_rank": scores.get("consistency_rank", 0),
                "speed_rank": scores.get("speed_rank", 0),
                "overall_tradability_rank": scores.get("overall_tradability_rank", 0),
                
                # Scores for reference
                "trade_frequency_score": round(scores.get("trade_frequency_score", 0), 2),
                "consistency_score": round(scores.get("consistency_score", 0), 2),
                "speed_score": round(scores.get("speed_score", 0), 2),
                "overall_tradability_score": round(scores.get("overall_tradability", 0), 2),
                
                # Raw metrics for context
                "bars_per_hour": round(scores.get("bars_per_hour", 0), 4),
                "avg_trades_per_bar": round(scores.get("avg_trades_per_bar", 0), 0),
                "completion_rate": round(scores.get("completion_rate", 0), 4),
                "duration_mean_seconds": round(scores.get("duration_mean_seconds", 0), 0),
                "total_bars": scores.get("total_bars", 0),
                "total_trades": scores.get("total_trades", 0)
            }
            rankings_data.append(ranking_row)
        
        # Save comprehensive rankings CSV (sorted by overall rank)
        rankings_df = pd.DataFrame(rankings_data).sort_values("overall_tradability_rank")
        comprehensive_csv = output_dir / f"comprehensive_tradability_rankings_{self.execution_timestamp}.csv"
        rankings_df.to_csv(comprehensive_csv, index=False)
        
        # === CREATE INDIVIDUAL RANKING CSVs ===
        
        ranking_categories = {
            "trade_frequency": ("trade_frequency_rank", "trade_frequency_score"),
            "consistency": ("consistency_rank", "consistency_score"),
            "speed": ("speed_rank", "speed_score")
        }
        
        individual_csv_files = {}
        
        for category, (rank_col, score_col) in ranking_categories.items():
            # Create individual ranking CSV
            individual_data = []
            for symbol, scores in calculated_scores.items():
                individual_data.append({
                    "rank": scores.get(rank_col, 0),
                    "symbol": symbol,
                    "score": round(scores.get(score_col, 0), 2),
                    "bars_per_hour": round(scores.get("bars_per_hour", 0), 4),
                    "total_bars": scores.get("total_bars", 0),
                    "total_trades": scores.get("total_trades", 0)
                })
            
            individual_df = pd.DataFrame(individual_data).sort_values("rank")
            individual_csv = output_dir / f"{category}_ranking_{self.execution_timestamp}.csv"
            individual_df.to_csv(individual_csv, index=False)
            individual_csv_files[category] = str(individual_csv)
        
        # Save discovery file
        discovery_file = output_dir / report["claude_code_discovery"]["discovery_file"]
        discovery_data = {
            "type": "comprehensive_tradability_rankings",
            "execution_timestamp": self.execution_timestamp,
            "files": {
                "comprehensive_report": str(report_file),
                "comprehensive_rankings_csv": str(comprehensive_csv),
                "individual_ranking_csvs": individual_csv_files
            },
            "ranking_categories": {
                "core_4": ["volatility", "liquidity", "efficiency", "consistency"],
                "creative_additional": ["speed", "predictability", "price_discovery", "market_activity", "processing_efficiency"]
            },
            "quick_results": {
                "most_tradable_overall": report["executive_summary"]["most_tradable_symbol"],
                "least_tradable_overall": report["executive_summary"]["least_tradable_symbol"],
                "symbols_analyzed": report["analysis_metadata"]["symbols_analyzed"],
                "ranking_logic": "1 = best performance, higher rank = lower performance"
            },
            "claude_code_analysis_ready": True,
            "csv_sortable": True
        }
        
        with open(discovery_file, 'w') as f:
            json.dump(discovery_data, f, indent=2)
        
        print(f"💾 Analysis saved:")
        print(f"   📊 Comprehensive report: {report_file}")
        print(f"   📈 Comprehensive rankings CSV: {comprehensive_csv}")
        print(f"   🎯 Individual ranking CSVs: {len(individual_csv_files)} files created")
        print(f"   🔍 Discovery file: {discovery_file}")
        print(f"   📋 Ranking logic: Rank 1 = best, higher rank = lower performance")
    
    def run_analysis(self) -> Dict[str, Any]:
        """Run complete tradability analysis pipeline"""
        
        print("🚀 Starting Premium Symbol Tradability Analysis")
        print("=" * 80)
        
        # Step 1: Discover completed symbols
        completed_symbols = self.discover_completed_symbols()
        if not completed_symbols:
            print("❌ No completed symbol analyses found!")
            return {}
        
        # Step 2: Extract volatility metrics
        print(f"\n📈 Extracting volatility metrics from {len(completed_symbols)} symbols...")
        symbols_metrics = {}
        
        for symbol_data in completed_symbols:
            symbol = symbol_data["symbol"]
            metrics = self.extract_volatility_metrics(symbol_data)
            if metrics:
                symbols_metrics[symbol] = metrics
        
        if not symbols_metrics:
            print("❌ No valid volatility metrics extracted!")
            return {}
        
        print(f"✅ Extracted metrics for {len(symbols_metrics)} symbols")
        
        # Step 3: Calculate all scores and individual rankings
        tradability_scores = self.calculate_all_scores_and_rankings(symbols_metrics)
        
        # Step 4: Generate comprehensive report
        report = self.generate_tradability_report(symbols_metrics, tradability_scores)
        
        # Step 5: Save results
        self.save_analysis_results(report)
        
        print(f"\n🎉 Tradability analysis completed!")
        print(f"🏆 Most tradable symbol: {report['executive_summary']['most_tradable_symbol']}")
        print(f"📊 Average tradability score: {report['executive_summary']['average_tradability_score']:.2f}")
        
        return report

def main():
    """Main execution function"""
    analyzer = TradabilityAnalyzer()
    
    try:
        results = analyzer.run_analysis()
        return 0 if results else 1
        
    except Exception as e:
        print(f"\n❌ Tradability analysis failed: {str(e)}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    import sys
    sys.exit(main())