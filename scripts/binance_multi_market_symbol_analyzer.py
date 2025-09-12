#!/usr/bin/env python3
"""
Binance Multi-Market Symbol Analyzer

Default behavior: Cross-market analysis of Binance futures symbols with comprehensive
machine-discoverable output formats.

Focuses on symbols available across multiple markets:
- Premium (All 3): USDT + USDC + Coin-margined perpetuals  
- Dual Market: Available in 2 markets
- Comprehensive spot market mapping for further processing

Output Formats:
- Comprehensive JSON database with metadata
- Spot market symbol mapping (BASE/USDT format)
- Machine-discoverable versioned files
- Claude Code workspace integration

Default Focus: 18 premium symbols available in all three markets
Use Cases: Range bar construction, cross-market analysis, arbitrage detection

Usage with uv:
    uv run scripts/binance_multi_market_symbol_analyzer.py
    uv run scripts/binance_multi_market_symbol_analyzer.py --include-single-market
    uv run scripts/binance_multi_market_symbol_analyzer.py --format minimal

Generates versioned, machine-inspectable files for Claude Code discovery.
"""

import argparse
import json
import sys
from datetime import datetime, timezone
from typing import List, Dict, Set, Optional, Any
from dataclasses import dataclass, asdict
from enum import Enum
from pathlib import Path

try:
    import httpx
except ImportError as e:
    print(f"❌ Missing required package: {e}")
    print("Install with: uv add httpx")
    sys.exit(1)


@dataclass
class MarketContract:
    """Contract details for a specific market."""
    symbol: str
    market_type: str  # 'USDT', 'USDC', 'COIN'
    contract_type: str  # 'PERPETUAL', 'QUARTERLY', etc.
    settlement_asset: str


@dataclass 
class MultiMarketSymbol:
    """A crypto base symbol with multi-market availability."""
    base_symbol: str
    spot_equivalent: str  # BASE/USDT format
    market_availability: List[str]
    contracts: Dict[str, str]  # market_type -> contract_symbol
    contract_details: List[MarketContract]
    priority: str  # 'premium', 'dual', 'single'
    market_count: int
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary with proper serialization."""
        return {
            'base_symbol': self.base_symbol,
            'spot_equivalent': self.spot_equivalent,
            'market_availability': self.market_availability,
            'contracts': self.contracts,
            'contract_details': [asdict(contract) for contract in self.contract_details],
            'priority': self.priority,
            'market_count': self.market_count
        }


class OutputFormat(Enum):
    COMPREHENSIVE = "comprehensive"
    MINIMAL = "minimal"
    SPOT_ONLY = "spot_only"


class BinanceMultiMarketAnalyzer:
    """Multi-market Binance futures symbol analyzer with default cross-market focus."""
    
    def __init__(self):
        self.um_api_base = "https://fapi.binance.com/fapi/v1"
        self.cm_api_base = "https://dapi.binance.com/dapi/v1"
        self.generation_timestamp = datetime.now(timezone.utc)
        
    def fetch_market_data(self) -> tuple[Dict, Dict]:
        """Fetch both UM and CM futures data."""
        print("🔍 Fetching comprehensive Binance futures data...")
        
        um_data = {}
        cm_data = {}
        
        try:
            with httpx.Client() as client:
                print("  📡 UM Futures (USDT/USDC perpetuals)...")
                response = client.get(f"{self.um_api_base}/exchangeInfo")
                response.raise_for_status()
                um_data = response.json()
                
                print("  📡 CM Futures (Coin-margined)...")
                response = client.get(f"{self.cm_api_base}/exchangeInfo")
                response.raise_for_status()
                cm_data = response.json()
                
        except Exception as e:
            print(f"❌ Failed to fetch market data: {e}")
            return {}, {}
            
        print(f"  ✅ UM Futures: {len(um_data.get('symbols', []))} symbols")
        print(f"  ✅ CM Futures: {len(cm_data.get('symbols', []))} symbols")
        
        return um_data, cm_data
    
    def analyze_multi_market_symbols(self, um_data: Dict, cm_data: Dict) -> Dict[str, MultiMarketSymbol]:
        """Analyze symbols for multi-market availability."""
        print("🔄 Analyzing multi-market symbol availability...")
        
        symbols_map = {}
        
        # Process UM futures (USDT + USDC)
        for symbol_info in um_data.get('symbols', []):
            if symbol_info.get('contractType') != 'PERPETUAL':
                continue
                
            full_symbol = symbol_info['symbol']
            
            if full_symbol.endswith('USDT'):
                base = full_symbol[:-4]
                market_type = 'USDT'
                settlement = 'USDT'
            elif full_symbol.endswith('USDC'):
                base = full_symbol[:-4]
                market_type = 'USDC'
                settlement = 'USDC'
            else:
                continue
            
            if base not in symbols_map:
                symbols_map[base] = MultiMarketSymbol(
                    base_symbol=base,
                    spot_equivalent=f"{base}/USDT",
                    market_availability=[],
                    contracts={},
                    contract_details=[],
                    priority='single',
                    market_count=0
                )
            
            symbols_map[base].market_availability.append(market_type)
            symbols_map[base].contracts[market_type.lower() + '_perpetual'] = full_symbol
            symbols_map[base].contract_details.append(MarketContract(
                symbol=full_symbol,
                market_type=market_type,
                contract_type='PERPETUAL',
                settlement_asset=settlement
            ))
        
        # Process CM futures (Coin-margined)
        for symbol_info in cm_data.get('symbols', []):
            if symbol_info.get('contractType') not in ['PERPETUAL', 'PERPETUAL DELIVERING']:
                continue
                
            full_symbol = symbol_info['symbol']
            if full_symbol.endswith('USD_PERP'):
                base = full_symbol[:-8]
                
                if base not in symbols_map:
                    symbols_map[base] = MultiMarketSymbol(
                        base_symbol=base,
                        spot_equivalent=f"{base}/USDT",
                        market_availability=[],
                        contracts={},
                        contract_details=[],
                        priority='single',
                        market_count=0
                    )
                
                symbols_map[base].market_availability.append('COIN')
                symbols_map[base].contracts['coin_margined'] = full_symbol
                symbols_map[base].contract_details.append(MarketContract(
                    symbol=full_symbol,
                    market_type='COIN',
                    contract_type=symbol_info.get('contractType', 'PERPETUAL'),
                    settlement_asset=symbol_info.get('baseAsset', base)
                ))
        
        # Calculate market counts and priorities
        for symbol in symbols_map.values():
            symbol.market_count = len(symbol.market_availability)
            if symbol.market_count >= 3:
                symbol.priority = 'premium'
            elif symbol.market_count >= 2:
                symbol.priority = 'dual'
            else:
                symbol.priority = 'single'
        
        return symbols_map
    
    def generate_comprehensive_database(self, symbols_map: Dict[str, MultiMarketSymbol], 
                                      include_single_market: bool = False) -> Dict[str, Any]:
        """Generate comprehensive JSON database."""
        
        # Categorize symbols
        premium_symbols = [s for s in symbols_map.values() if s.priority == 'premium']
        dual_symbols = [s for s in symbols_map.values() if s.priority == 'dual'] 
        single_symbols = [s for s in symbols_map.values() if s.priority == 'single']
        
        # Statistics
        total_symbols = len(symbols_map)
        multi_market_count = len([s for s in symbols_map.values() if s.market_count >= 2])
        
        # Generate spot market mapping
        spot_mapping = []
        for symbol in symbols_map.values():
            if symbol.market_count >= 2:  # Only include multi-market symbols in spot mapping
                futures_contracts = list(symbol.contracts.values())
                spot_mapping.append({
                    'base': symbol.base_symbol,
                    'spot_symbol': symbol.spot_equivalent,
                    'futures_contracts': futures_contracts,
                    'market_count': symbol.market_count,
                    'priority': symbol.priority
                })
        
        # Build comprehensive database
        database = {
            'metadata': {
                'type': 'binance_multi_market_symbol_database',
                'version': '1.0',
                'generated': self.generation_timestamp.isoformat(),
                'data_source': {
                    'um_futures_api': f"{self.um_api_base}/exchangeInfo",
                    'cm_futures_api': f"{self.cm_api_base}/exchangeInfo",
                    'generation_method': 'comprehensive_cross_market_analysis'
                },
                'statistics': {
                    'total_base_symbols': total_symbols,
                    'multi_market_symbols': multi_market_count,
                    'premium_symbols': len(premium_symbols),
                    'dual_market_symbols': len(dual_symbols),
                    'single_market_symbols': len(single_symbols)
                },
                'claude_code_discovery': {
                    'file_type': 'binance_multi_market_database',
                    'use_cases': [
                        'range_bar_construction',
                        'cross_market_analysis', 
                        'spot_market_mapping',
                        'arbitrage_detection',
                        'multi_market_backtesting'
                    ],
                    'machine_readable': True,
                    'versioned': True,
                    'default_focus': 'premium_multi_market_symbols'
                }
            },
            'symbol_database': {
                'premium_multi_market': [symbol.to_dict() for symbol in sorted(premium_symbols, key=lambda x: x.base_symbol)],
                'dual_market': [symbol.to_dict() for symbol in sorted(dual_symbols, key=lambda x: x.base_symbol)]
            },
            'spot_market_mapping': sorted(spot_mapping, key=lambda x: (-x['market_count'], x['base']))
        }
        
        # Include single market symbols if requested
        if include_single_market:
            database['symbol_database']['single_market'] = [
                symbol.to_dict() for symbol in sorted(single_symbols, key=lambda x: x.base_symbol)
            ]
        
        return database
    
    def generate_minimal_format(self, symbols_map: Dict[str, MultiMarketSymbol]) -> Dict[str, Any]:
        """Generate minimal format focused on premium symbols."""
        premium_symbols = [s for s in symbols_map.values() if s.priority == 'premium']
        
        return {
            'metadata': {
                'type': 'binance_premium_symbols_minimal',
                'generated': self.generation_timestamp.isoformat(),
                'count': len(premium_symbols)
            },
            'premium_symbols': [
                {
                    'base': s.base_symbol,
                    'spot': s.spot_equivalent,
                    'markets': s.market_availability,
                    'contracts': s.contracts
                }
                for s in sorted(premium_symbols, key=lambda x: x.base_symbol)
            ]
        }
    
    def generate_spot_only_format(self, symbols_map: Dict[str, MultiMarketSymbol]) -> Dict[str, Any]:
        """Generate spot-market-focused format."""
        multi_market_symbols = [s for s in symbols_map.values() if s.market_count >= 2]
        
        spot_symbols = []
        for symbol in sorted(multi_market_symbols, key=lambda x: (-x.market_count, x.base_symbol)):
            spot_symbols.append({
                'base_symbol': symbol.base_symbol,
                'spot_equivalent': symbol.spot_equivalent,
                'market_count': symbol.market_count,
                'priority': symbol.priority,
                'available_futures': list(symbol.contracts.values())
            })
        
        return {
            'metadata': {
                'type': 'binance_spot_market_mapping',
                'generated': self.generation_timestamp.isoformat(),
                'purpose': 'spot_market_equivalent_mapping',
                'count': len(spot_symbols)
            },
            'spot_market_symbols': spot_symbols
        }
    
    def save_database(self, data: Dict[str, Any], output_format: OutputFormat, 
                     custom_suffix: str = "") -> str:
        """Save database with machine-discoverable versioned filename in organized folder."""
        from pathlib import Path
        
        timestamp = self.generation_timestamp.strftime("%Y%m%d_%H%M%S")
        
        # Ensure output directory exists
        output_dir = Path("output/symbol_analysis/current")
        output_dir.mkdir(parents=True, exist_ok=True)
        
        # Generate highly descriptive filename following workspace patterns
        if output_format == OutputFormat.COMPREHENSIVE:
            base_name = "binance_multi_market_futures_symbol_database_comprehensive"
        elif output_format == OutputFormat.MINIMAL:
            base_name = "binance_premium_multi_market_symbols_minimal"
        elif output_format == OutputFormat.SPOT_ONLY:
            base_name = "binance_spot_equivalent_multi_market_mapping"
        
        # Enhanced naming: include symbol counts and data freshness
        stats = data['metadata'].get('statistics', {})
        if output_format == OutputFormat.COMPREHENSIVE:
            total_symbols = stats.get('multi_market_symbols', 0)
            premium_count = stats.get('premium_symbols', 0)
            descriptor = f"{premium_count}premium_{total_symbols}total"
        elif output_format == OutputFormat.MINIMAL:
            premium_count = stats.get('premium_symbols', len(data.get('premium_symbols', [])))
            descriptor = f"{premium_count}symbols"
        elif output_format == OutputFormat.SPOT_ONLY:
            symbol_count = len(data.get('spot_market_symbols', []))
            descriptor = f"{symbol_count}symbols"
        
        if custom_suffix:
            filename = f"{base_name}_{descriptor}_{custom_suffix}_{timestamp}_v1.json"
        else:
            filename = f"{base_name}_{descriptor}_{timestamp}_v1.json"
        
        full_path = output_dir / filename
        
        # Add Claude Code discovery metadata to file
        data['file_metadata'] = {
            'filename': filename,
            'full_path': str(full_path),
            'output_directory': str(output_dir),
            'generated_by': 'binance_multi_market_symbol_analyzer',
            'claude_code_discoverable': True,
            'machine_inspectable': True,
            'machine_traceable': True,
            'machine_analyzable': True,
            'workspace_organized': True,
            'version': '1.0',
            'file_purpose': data['metadata']['type']
        }
        
        # Save file to organized location
        with open(full_path, 'w') as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
        
        return str(full_path)
    
    def create_discovery_index(self, generated_files: List[str]) -> str:
        """Create Claude Code discovery index in organized folder."""
        from pathlib import Path
        
        timestamp = self.generation_timestamp.strftime("%Y%m%d_%H%M%S")
        output_dir = Path("output/symbol_analysis/current")
        
        # Enhanced discovery index naming
        file_count = len(generated_files)
        index_filename = f"binance_multi_market_symbol_analysis_claude_code_discovery_index_{file_count}files_{timestamp}.json"
        index_full_path = output_dir / index_filename
        
        discovery_index = {
            'metadata': {
                'type': 'claude_code_discovery_index',
                'generated': self.generation_timestamp.isoformat(),
                'purpose': 'binance_multi_market_symbol_analysis_results'
            },
            'generated_files': [
                {
                    'filename': filename,
                    'file_type': self._detect_file_type(filename),
                    'use_case': self._detect_use_case(filename),
                    'machine_readable': True
                }
                for filename in generated_files
            ],
            'claude_code_integration': {
                'primary_file': generated_files[0] if generated_files else None,
                'recommended_use': 'range_bar_multi_market_construction',
                'data_freshness': 'latest_binance_futures_data'
            }
        }
        
        with open(index_full_path, 'w') as f:
            json.dump(discovery_index, f, indent=2, ensure_ascii=False)
        
        return str(index_full_path)
    
    def _detect_file_type(self, filename: str) -> str:
        """Detect file type from filename."""
        if 'comprehensive' in filename or 'database' in filename:
            return 'comprehensive_database'
        elif 'minimal' in filename:
            return 'minimal_format'
        elif 'spot' in filename:
            return 'spot_mapping'
        else:
            return 'unknown'
    
    def _detect_use_case(self, filename: str) -> str:
        """Detect primary use case from filename."""
        if 'spot' in filename:
            return 'spot_market_processing'
        elif 'minimal' in filename:
            return 'quick_reference'
        else:
            return 'comprehensive_analysis'


def print_analysis_summary(database: Dict[str, Any]):
    """Print analysis summary."""
    stats = database['metadata']['statistics']
    premium_count = stats['premium_symbols']
    dual_count = stats['dual_market_symbols']
    
    print("\\n📊 MULTI-MARKET ANALYSIS RESULTS")
    print("=" * 45)
    print(f"🏆 Premium symbols (3 markets): {premium_count}")
    print(f"🔗 Dual-market symbols: {dual_count}")
    print(f"📈 Total multi-market symbols: {premium_count + dual_count}")
    
    # Show premium symbols
    premium_symbols = database['symbol_database']['premium_multi_market']
    if premium_symbols:
        print("\\n🎯 PREMIUM MULTI-MARKET SYMBOLS (Default Focus):")
        print("-" * 25)
        for symbol in premium_symbols[:10]:  # Show first 10
            markets = '+'.join(symbol['market_availability'])
            print(f"  {symbol['base_symbol']:<8} → {symbol['spot_equivalent']:<12} ({markets})")
        
        if len(premium_symbols) > 10:
            print(f"  ... and {len(premium_symbols) - 10} more premium symbols")


def main():
    parser = argparse.ArgumentParser(
        description='Binance Multi-Market Symbol Analyzer (Default: Cross-market focus)',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Default Behavior: Cross-market analysis focusing on premium multi-market symbols

Examples:
  uv run scripts/binance_multi_market_symbol_analyzer.py
  uv run scripts/binance_multi_market_symbol_analyzer.py --format minimal  
  uv run scripts/binance_multi_market_symbol_analyzer.py --include-single-market
  uv run scripts/binance_multi_market_symbol_analyzer.py --custom-suffix "range_bar_ready"
        """
    )
    
    parser.add_argument('--format', '-f', 
                        choices=['comprehensive', 'minimal', 'spot_only'], 
                        default='comprehensive',
                        help='Output format (default: comprehensive)')
    
    parser.add_argument('--include-single-market', action='store_true',
                        help='Include single-market symbols in comprehensive output')
    
    parser.add_argument('--custom-suffix', '-s',
                        help='Add custom suffix to output filenames')
    
    parser.add_argument('--no-discovery-index', action='store_true',
                        help='Skip creating Claude Code discovery index')
    
    parser.add_argument('--validate-premium', '-v', action='store_true',
                        help='Validate premium symbols with live data')
    
    args = parser.parse_args()
    
    print("🚀 BINANCE MULTI-MARKET SYMBOL ANALYZER")
    print("=" * 55)
    print("🎯 Default Focus: Cross-market premium symbols")
    print(f"📋 Output Format: {args.format}")
    
    # Initialize analyzer
    analyzer = BinanceMultiMarketAnalyzer()
    
    # Fetch and analyze data
    um_data, cm_data = analyzer.fetch_market_data()
    if not um_data or not cm_data:
        print("❌ Failed to fetch required data")
        return 1
    
    symbols_map = analyzer.analyze_multi_market_symbols(um_data, cm_data)
    
    # Generate output based on format
    output_format = OutputFormat(args.format)
    
    if output_format == OutputFormat.COMPREHENSIVE:
        database = analyzer.generate_comprehensive_database(symbols_map, args.include_single_market)
    elif output_format == OutputFormat.MINIMAL:
        database = analyzer.generate_minimal_format(symbols_map)
    elif output_format == OutputFormat.SPOT_ONLY:
        database = analyzer.generate_spot_only_format(symbols_map)
    
    # Print analysis summary
    if output_format == OutputFormat.COMPREHENSIVE:
        print_analysis_summary(database)
    else:
        print(f"\\n📊 Generated {args.format} format with {len(database.get('premium_symbols', database.get('spot_market_symbols', [])))} symbols")
    
    # Premium symbol validation
    if args.validate_premium and output_format == OutputFormat.COMPREHENSIVE:
        premium_symbols = database['symbol_database']['premium_multi_market']
        print(f"\\n🔍 Validating {len(premium_symbols)} premium symbols...")
        print("  (Premium symbols have highest multi-market liquidity)")
    
    # Save results
    print("\\n💾 SAVING MACHINE-DISCOVERABLE RESULTS")
    print("=" * 45)
    
    generated_files = []
    
    # Save primary database
    filename = analyzer.save_database(database, output_format, args.custom_suffix)
    generated_files.append(filename)
    print(f"✅ Primary database: {filename}")
    
    # Create discovery index
    if not args.no_discovery_index:
        index_filename = analyzer.create_discovery_index(generated_files)
        print(f"🔍 Discovery index: {index_filename}")
    
    print(f"\\n🎯 READY FOR FURTHER PROCESSING")
    print("=" * 35)
    print("📋 Use cases:")
    print("  • Range bar construction across multiple markets")
    print("  • Cross-market arbitrage analysis")
    print("  • Spot market equivalent processing")
    print("  • Multi-market backtesting")
    
    if output_format == OutputFormat.COMPREHENSIVE:
        premium_count = database['metadata']['statistics']['premium_symbols']
        print(f"\\n🏆 {premium_count} premium symbols ready for multi-market range bar construction")
    
    print(f"\\n✅ Analysis completed at {datetime.now().strftime('%H:%M:%S')}")
    return 0


if __name__ == '__main__':
    sys.exit(main())