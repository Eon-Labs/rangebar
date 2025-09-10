"""
Command-line interface for range bar data operations.

Provides complete workflow from data fetching to range bar generation
with Parquet storage for high-performance processing.
"""

import asyncio
import logging
import sys
from pathlib import Path
from typing import Optional
import click
from decimal import Decimal

from .data_fetcher import fetch_um_futures_aggtrades
from .range_bars import iter_range_bars_from_aggtrades
from .parquet_converter import ParquetConverter


# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(sys.stdout)
    ]
)
logger = logging.getLogger(__name__)


@click.group()
@click.option('--verbose', '-v', is_flag=True, help='Enable verbose logging')
def cli(verbose: bool):
    """
    Range Bar CLI - Non-lookahead bias range bar construction from Binance UM Futures data.
    
    This tool fetches aggTrades data from Binance UM Futures and constructs range bars
    with proper non-lookahead bias validation using high-performance Rust algorithms.
    """
    if verbose:
        logging.getLogger().setLevel(logging.DEBUG)
        logger.debug("Verbose logging enabled")


@cli.command()
@click.argument('symbol', type=str)
@click.argument('start_date', type=str)  
@click.argument('end_date', type=str)
@click.option('--output-dir', '-o', type=click.Path(), default='./data',
              help='Output directory for data files (default: ./data)')
@click.option('--format', '-f', type=click.Choice(['parquet', 'csv']), default='parquet',
              help='Output format (default: parquet)')
@click.option('--compression', '-c', type=click.Choice(['snappy', 'gzip', 'lz4', 'brotli']), 
              default='snappy', help='Compression algorithm (default: snappy)')
def fetch(symbol: str, start_date: str, end_date: str, output_dir: str, 
         format: str, compression: str):
    """
    Fetch aggTrades data from Binance UM Futures.
    
    SYMBOL: Trading symbol (e.g., BTCUSDT, ETHUSDT)
    START_DATE: Start date in YYYY-MM-DD format
    END_DATE: End date in YYYY-MM-DD format
    
    Examples:
        rangebar fetch BTCUSDT 2024-01-01 2024-01-02
        rangebar fetch ETHUSDT 2024-01-01 2024-01-01 --format csv
    """
    asyncio.run(_fetch_data(symbol, start_date, end_date, output_dir, format, compression))


async def _fetch_data(symbol: str, start_date: str, end_date: str, 
                     output_dir: str, format: str, compression: str):
    """Internal async function for data fetching."""
    try:
        click.echo(f"🚀 Fetching {symbol} aggTrades from {start_date} to {end_date}")
        
        # Fetch data
        aggtrades = await fetch_um_futures_aggtrades(symbol, start_date, end_date)
        
        click.echo(f"✅ Fetched {len(aggtrades):,} aggTrades")
        
        # Setup output directory
        output_path = Path(output_dir)
        output_path.mkdir(parents=True, exist_ok=True)
        
        if format == 'parquet':
            # Save as Parquet
            parquet_path = output_path / f"{symbol}_aggtrades_{start_date}_{end_date}.parquet"
            converter = ParquetConverter(compression=compression)
            converter.aggtrades_to_parquet(aggtrades, parquet_path, symbol)
            
            file_size = parquet_path.stat().st_size
            click.echo(f"💾 Saved to Parquet: {parquet_path} ({file_size:,} bytes)")
            
        else:  # CSV format
            import csv
            csv_path = output_path / f"{symbol}_aggtrades_{start_date}_{end_date}.csv"
            
            with open(csv_path, 'w', newline='') as f:
                writer = csv.writer(f)
                writer.writerow(['agg_trade_id', 'price', 'quantity', 'first_trade_id', 
                               'last_trade_id', 'timestamp', 'is_buyer_maker'])
                
                for trade in aggtrades:
                    writer.writerow([
                        trade.agg_trade_id, 
                        str(trade.price), 
                        str(trade.quantity),
                        trade.first_trade_id,
                        trade.last_trade_id,
                        trade.timestamp,
                        trade.is_buyer_maker
                    ])
            
            file_size = csv_path.stat().st_size
            click.echo(f"💾 Saved to CSV: {csv_path} ({file_size:,} bytes)")
        
        click.echo("🎉 Data fetch completed successfully!")
        
    except Exception as e:
        click.echo(f"❌ Error fetching data: {e}", err=True)
        sys.exit(1)


@cli.command()
@click.argument('symbol', type=str)
@click.argument('start_date', type=str)
@click.argument('end_date', type=str)
@click.option('--threshold', '-t', type=float, default=0.8,
              help='Range bar threshold percentage (default: 0.8)')
@click.option('--output-dir', '-o', type=click.Path(), default='./data',
              help='Output directory for range bars (default: ./data)')
@click.option('--input-format', type=click.Choice(['fetch', 'parquet', 'csv']), 
              default='fetch', help='Input data source (default: fetch)')
@click.option('--input-file', type=click.Path(exists=True),
              help='Input file path (for parquet/csv formats)')
@click.option('--compression', '-c', type=click.Choice(['snappy', 'gzip', 'lz4', 'brotli']),
              default='snappy', help='Output compression (default: snappy)')
@click.option('--max-trades', type=int, help='Limit number of trades processed (for testing)')
def generate(symbol: str, start_date: str, end_date: str, threshold: float,
            output_dir: str, input_format: str, input_file: Optional[str], 
            compression: str, max_trades: Optional[int]):
    """
    Generate range bars from aggTrades data.
    
    SYMBOL: Trading symbol (e.g., BTCUSDT, ETHUSDT)  
    START_DATE: Start date in YYYY-MM-DD format
    END_DATE: End date in YYYY-MM-DD format
    
    Examples:
        rangebar generate BTCUSDT 2024-01-01 2024-01-02
        rangebar generate BTCUSDT 2024-01-01 2024-01-01 --threshold 1.0
        rangebar generate BTCUSDT 2024-01-01 2024-01-01 --input-format parquet --input-file data/BTCUSDT.parquet
    """
    asyncio.run(_generate_range_bars(
        symbol, start_date, end_date, threshold, output_dir, 
        input_format, input_file, compression, max_trades
    ))


async def _generate_range_bars(symbol: str, start_date: str, end_date: str, threshold: float,
                              output_dir: str, input_format: str, input_file: Optional[str],
                              compression: str, max_trades: Optional[int]):
    """Internal async function for range bar generation."""
    try:
        click.echo(f"🏗️ Generating range bars for {symbol} ({threshold}% threshold)")
        
        # Load data based on input format
        if input_format == 'fetch':
            click.echo("📥 Fetching fresh data from Binance...")
            aggtrades = await fetch_um_futures_aggtrades(symbol, start_date, end_date)
            
        elif input_format == 'parquet':
            if not input_file:
                raise ValueError("--input-file required for parquet format")
            
            click.echo(f"📥 Loading from Parquet: {input_file}")
            converter = ParquetConverter()
            table = converter.load_aggtrades_from_parquet(Path(input_file), symbol_filter=symbol)
            
            # Convert PyArrow table back to AggTrade objects
            from .range_bars import AggTrade
            aggtrades = []
            for row in table.to_pylist():
                # Convert back to expected format
                data = {
                    'a': row['agg_trade_id'],
                    'p': str(row['price']),
                    'q': str(row['quantity']),
                    'f': row['first_trade_id'],
                    'l': row['last_trade_id'],
                    'T': row['timestamp'].timestamp() * 1000,  # Convert to milliseconds
                    'm': row['is_buyer_maker']
                }
                aggtrades.append(AggTrade(data))
                
        elif input_format == 'csv':
            if not input_file:
                raise ValueError("--input-file required for csv format")
                
            click.echo(f"📥 Loading from CSV: {input_file}")
            import csv
            from .range_bars import AggTrade
            
            aggtrades = []
            with open(input_file, 'r') as f:
                reader = csv.DictReader(f)
                for row in reader:
                    data = {
                        'a': int(row['agg_trade_id']),
                        'p': row['price'],
                        'q': row['quantity'],
                        'f': int(row['first_trade_id']),
                        'l': int(row['last_trade_id']),
                        'T': int(row['timestamp']),
                        'm': row['is_buyer_maker'].lower() == 'true'
                    }
                    aggtrades.append(AggTrade(data))
        
        # Limit trades if specified
        if max_trades and len(aggtrades) > max_trades:
            click.echo(f"⚠️ Limiting to {max_trades:,} trades (from {len(aggtrades):,})")
            aggtrades = aggtrades[:max_trades]
        
        click.echo(f"📊 Processing {len(aggtrades):,} aggTrades")
        
        # Generate range bars
        pct_threshold = Decimal(str(threshold / 100.0))  # Convert percentage to decimal
        range_bars = list(iter_range_bars_from_aggtrades(aggtrades, pct=pct_threshold))
        
        if not range_bars:
            click.echo("⚠️ No range bars generated - threshold may be too large for the data range")
            return
        
        click.echo(f"✅ Generated {len(range_bars):,} range bars")
        
        # Setup output directory
        output_path = Path(output_dir)
        output_path.mkdir(parents=True, exist_ok=True)
        
        # Save range bars to Parquet
        rb_parquet_path = output_path / f"{symbol}_range_bars_{threshold}pct_{start_date}_{end_date}.parquet"
        threshold_bps = int(threshold * 100)  # Convert to basis points
        
        converter = ParquetConverter(compression=compression)
        converter.range_bars_to_parquet(range_bars, rb_parquet_path, symbol, threshold_bps)
        
        file_size = rb_parquet_path.stat().st_size
        click.echo(f"💾 Saved range bars: {rb_parquet_path} ({file_size:,} bytes)")
        
        # Show summary statistics
        if range_bars:
            first_bar = range_bars[0]
            last_bar = range_bars[-1]
            total_volume = sum(float(str(bar['volume'])) for bar in range_bars)
            total_trades = sum(int(bar['trade_count']) for bar in range_bars)
            
            click.echo("\n📈 Range Bar Summary:")
            click.echo(f"   Bars Generated: {len(range_bars):,}")
            click.echo(f"   Threshold: {threshold}% ({threshold_bps} bps)")
            click.echo(f"   Time Range: {first_bar['open_time']} → {last_bar['close_time']}")
            click.echo(f"   Price Range: {first_bar['open']} → {last_bar['close']}")
            click.echo(f"   Total Volume: {total_volume:,.8f}")
            click.echo(f"   Total Trades: {total_trades:,}")
            click.echo(f"   Avg Trades/Bar: {total_trades / len(range_bars):.1f}")
        
        click.echo("\n🎉 Range bar generation completed successfully!")
        
    except Exception as e:
        click.echo(f"❌ Error generating range bars: {e}", err=True)
        sys.exit(1)


@cli.command()
@click.argument('parquet_file', type=click.Path(exists=True))
@click.option('--symbol', '-s', type=str, help='Filter by symbol')
@click.option('--threshold', '-t', type=int, help='Filter by threshold (basis points)')
@click.option('--limit', '-l', type=int, default=10, help='Limit output rows (default: 10)')
def inspect(parquet_file: str, symbol: Optional[str], threshold: Optional[int], limit: int):
    """
    Inspect Parquet files containing aggTrades or range bars.
    
    PARQUET_FILE: Path to the Parquet file to inspect
    
    Examples:
        rangebar inspect data/BTCUSDT_aggtrades.parquet
        rangebar inspect data/BTCUSDT_range_bars.parquet --limit 5
        rangebar inspect data/multi_symbol.parquet --symbol ETHUSDT
    """
    try:
        file_path = Path(parquet_file)
        converter = ParquetConverter()
        
        # Get file metadata
        file_info = converter.get_file_info(file_path)
        
        click.echo(f"📊 Inspecting: {file_path}")
        click.echo(f"   File Size: {file_info['file_size']:,} bytes")
        click.echo(f"   Rows: {file_info['num_rows']:,}")
        click.echo(f"   Columns: {file_info['num_columns']}")
        click.echo(f"   Row Groups: {file_info['num_row_groups']}")
        click.echo(f"   Compression: {file_info['compression']}")
        
        # Determine file type and load data
        try:
            # Try loading as range bars first (without filters initially)
            table = converter.load_range_bars_from_parquet(file_path)
            
            click.echo(f"\n📈 Range Bars Data (showing first {limit} rows):")
            click.echo("   open_time | close_time | open | high | low | close | volume | trades")
            click.echo("   " + "-" * 80)
            
            for i, row in enumerate(table.to_pylist()[:limit]):
                open_time = row['open_time']
                close_time = row['close_time']
                click.echo(f"   {open_time} | {close_time} | {row['open']:.4f} | {row['high']:.4f} | {row['low']:.4f} | {row['close']:.4f} | {row['volume']:.4f} | {row['trade_count']}")
                
        except:
            # Fall back to aggTrades (without filters initially)
            table = converter.load_aggtrades_from_parquet(file_path)
            
            click.echo(f"\n📊 aggTrades Data (showing first {limit} rows):")
            click.echo("   timestamp | agg_trade_id | price | quantity | is_buyer_maker")
            click.echo("   " + "-" * 70)
            
            for i, row in enumerate(table.to_pylist()[:limit]):
                timestamp = row['timestamp']
                click.echo(f"   {timestamp} | {row['agg_trade_id']} | {row['price']:.8f} | {row['quantity']:.8f} | {row['is_buyer_maker']}")
        
        click.echo(f"\n✅ Inspection completed ({table.num_rows:,} rows total)")
        
    except Exception as e:
        click.echo(f"❌ Error inspecting file: {e}", err=True)
        sys.exit(1)


@cli.command()
@click.option('--data-dir', type=click.Path(), default='./data',
              help='Data directory to clean (default: ./data)')
@click.option('--older-than', type=int, default=7,
              help='Remove files older than N days (default: 7)')
@click.option('--dry-run', is_flag=True, help='Show what would be deleted without deleting')
def cleanup(data_dir: str, older_than: int, dry_run: bool):
    """
    Clean up old data files.
    
    Examples:
        rangebar cleanup --older-than 30
        rangebar cleanup --dry-run
    """
    try:
        data_path = Path(data_dir)
        
        if not data_path.exists():
            click.echo(f"⚠️ Data directory does not exist: {data_path}")
            return
        
        import time
        cutoff_time = time.time() - (older_than * 24 * 60 * 60)
        
        old_files = []
        total_size = 0
        
        for file_path in data_path.rglob("*.parquet"):
            if file_path.is_file() and file_path.stat().st_mtime < cutoff_time:
                old_files.append(file_path)
                total_size += file_path.stat().st_size
        
        for file_path in data_path.rglob("*.csv"):
            if file_path.is_file() and file_path.stat().st_mtime < cutoff_time:
                old_files.append(file_path)
                total_size += file_path.stat().st_size
        
        if not old_files:
            click.echo(f"✅ No files older than {older_than} days found")
            return
        
        click.echo(f"🗑️ Found {len(old_files)} files older than {older_than} days ({total_size:,} bytes)")
        
        if dry_run:
            click.echo("\n📋 Files that would be deleted:")
            for file_path in old_files:
                file_age_days = (time.time() - file_path.stat().st_mtime) / (24 * 60 * 60)
                click.echo(f"   {file_path} (age: {file_age_days:.1f} days)")
        else:
            if click.confirm(f"\nDelete {len(old_files)} files?"):
                for file_path in old_files:
                    file_path.unlink()
                click.echo(f"✅ Deleted {len(old_files)} files ({total_size:,} bytes freed)")
            else:
                click.echo("❌ Cleanup cancelled")
        
    except Exception as e:
        click.echo(f"❌ Error during cleanup: {e}", err=True)
        sys.exit(1)


def main():
    """Entry point for the CLI."""
    cli()


if __name__ == '__main__':
    main()