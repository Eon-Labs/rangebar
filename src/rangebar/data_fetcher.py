"""
UM Futures data fetcher using binance_historical_data.

Fetches aggTrades data from Binance UM Futures with proper validation
and formatting for range bar construction.
"""

import asyncio
import logging
from datetime import datetime, timezone, date, timedelta
from pathlib import Path
from typing import List, Dict, Any, Optional
import json

from binance_historical_data import BinanceDataDumper

from .range_bars import AggTrade


logger = logging.getLogger(__name__)


class UMFuturesDataFetcher:
    """
    Fetches UM Futures aggTrades data from Binance Historical Data.
    
    Ensures data authenticity and proper formatting for range bar construction.
    """
    
    def __init__(self, 
                 data_directory: Optional[Path] = None,
                 asset_class: str = "um",  # UM futures only
                 data_type: str = "aggTrades"):
        """
        Initialize the data fetcher.
        
        Args:
            data_directory: Directory to store downloaded data
            asset_class: Asset class (um for UM Futures)
            data_type: Data type (aggTrades for aggregate trades)
        """
        if asset_class != "um":
            raise ValueError("Only UM Futures (asset_class='um') supported")
        if data_type != "aggTrades":
            raise ValueError("Only aggTrades data type supported")
            
        self.data_directory = data_directory or Path.home() / ".binance_data"
        self.asset_class = asset_class
        self.data_type = data_type
        
        # Initialize binance data dumper
        self.dumper = BinanceDataDumper(
            path_dir_where_to_dump=str(self.data_directory),
            asset_class=asset_class,
            data_type=data_type,
            data_frequency="daily"
        )
        
        logger.info(f"Initialized UM Futures data fetcher with directory: {self.data_directory}")
    
    def validate_symbol(self, symbol: str) -> str:
        """
        Validate and normalize symbol for UM Futures.
        
        Args:
            symbol: Trading symbol (e.g., 'BTCUSDT', 'btcusdt')
            
        Returns:
            Normalized symbol in uppercase
            
        Raises:
            ValueError: If symbol format is invalid
        """
        if not symbol or not isinstance(symbol, str):
            raise ValueError("Symbol must be a non-empty string")
            
        symbol = symbol.upper().strip()
        
        # Basic validation - should end with USDT for UM Futures
        if not symbol.endswith('USDT'):
            raise ValueError(f"UM Futures symbol must end with 'USDT', got: {symbol}")
            
        if len(symbol) < 6:  # Minimum: BTCUSDT
            raise ValueError(f"Symbol too short: {symbol}")
            
        return symbol
    
    def validate_date_range(self, start_date: str, end_date: str) -> tuple[str, str]:
        """
        Validate date range format and logical consistency.
        
        Args:
            start_date: Start date in YYYY-MM-DD format
            end_date: End date in YYYY-MM-DD format
            
        Returns:
            Tuple of validated (start_date, end_date)
            
        Raises:
            ValueError: If dates are invalid or inconsistent
        """
        try:
            start_dt = datetime.strptime(start_date, "%Y-%m-%d")
            end_dt = datetime.strptime(end_date, "%Y-%m-%d")
        except ValueError as e:
            raise ValueError(f"Invalid date format (expected YYYY-MM-DD): {e}")
            
        if start_dt > end_dt:
            raise ValueError(f"Start date {start_date} must be before or equal to end date {end_date}")
            
        # Check if dates are too far in the future
        now = datetime.now(timezone.utc)
        if start_dt.replace(tzinfo=timezone.utc) > now:
            raise ValueError(f"Start date {start_date} is in the future")
            
        return start_date, end_date
    
    async def fetch_aggtrades(self, 
                             symbol: str,
                             start_date: str, 
                             end_date: str) -> List[Path]:
        """
        Fetch aggTrades data for the specified symbol and date range.
        
        Args:
            symbol: Trading symbol (e.g., 'BTCUSDT')
            start_date: Start date in YYYY-MM-DD format
            end_date: End date in YYYY-MM-DD format
            
        Returns:
            List of paths to downloaded files
            
        Raises:
            ValueError: If parameters are invalid
            RuntimeError: If download fails
        """
        # Validate inputs
        symbol = self.validate_symbol(symbol)
        start_date, end_date = self.validate_date_range(start_date, end_date)
        
        logger.info(f"Fetching {symbol} aggTrades from {start_date} to {end_date}")
        
        try:
            # Convert string dates to datetime.date objects
            start_date_obj = datetime.strptime(start_date, "%Y-%m-%d").date()
            end_date_obj = datetime.strptime(end_date, "%Y-%m-%d").date()
            
            # Download data using binance_historical_data
            await asyncio.get_event_loop().run_in_executor(
                None, 
                self.dumper.dump_data,
                [symbol],  # tickers as list
                start_date_obj,  # date_start
                end_date_obj  # date_end
            )
            
            # Construct expected file paths 
            file_paths = []
            current_date = start_date_obj
            while current_date <= end_date_obj:
                date_str = current_date.strftime("%Y-%m-%d")
                file_path = (self.data_directory / 
                           "futures" / 
                           self.asset_class / 
                           "daily" / 
                           self.data_type / 
                           symbol / 
                           f"{symbol}-{self.data_type}-{date_str}.csv")
                
                if file_path.exists():
                    file_paths.append(file_path)
                else:
                    logger.warning(f"Expected file not found: {file_path}")
                
                # Move to next day
                current_date = current_date + timedelta(days=1)
                
            if not file_paths:
                raise RuntimeError(f"No data files found for {symbol} from {start_date} to {end_date}")
                
            logger.info(f"Found {len(file_paths)} files for {symbol}")
            return file_paths
            
        except Exception as e:
            raise RuntimeError(f"Failed to fetch data for {symbol}: {e}")
    
    def load_aggtrades_from_file(self, file_path: Path) -> List[AggTrade]:
        """
        Load aggTrades from a downloaded CSV file.
        
        Args:
            file_path: Path to the aggTrades CSV file
            
        Returns:
            List of AggTrade objects
            
        Raises:
            ValueError: If file format is invalid
            RuntimeError: If file cannot be read
        """
        if not file_path.exists():
            raise RuntimeError(f"File does not exist: {file_path}")
            
        logger.info(f"Loading aggTrades from {file_path}")
        
        try:
            # Read CSV file - binance_historical_data saves as CSV
            import csv
            
            aggtrades = []
            with open(file_path, 'r') as f:
                reader = csv.DictReader(f)
                
                for line_num, row in enumerate(reader, 2):  # Start at 2 for header line
                    try:
                        # Convert CSV row to aggTrade format expected by AggTrade class
                        data = {
                            'a': int(row['agg_trade_id']),  # agg_trade_id
                            'p': row['price'],  # price
                            'q': row['quantity'],  # quantity
                            'f': int(row['first_trade_id']),  # first_trade_id
                            'l': int(row['last_trade_id']),  # last_trade_id
                            'T': int(row['transact_time']),  # timestamp
                            'm': row['is_buyer_maker'].lower() == 'true'  # is_buyer_maker
                        }
                        
                        aggtrade = AggTrade(data)
                        aggtrades.append(aggtrade)
                        
                    except (ValueError, KeyError) as e:
                        raise ValueError(f"Invalid aggTrade data at line {line_num}: {e}")
            
            if not aggtrades:
                raise ValueError(f"No valid aggTrades found in {file_path}")
                
            # Validate data is sorted by timestamp and agg_trade_id
            for i in range(1, len(aggtrades)):
                prev = aggtrades[i-1]
                curr = aggtrades[i]
                
                if (curr.timestamp < prev.timestamp or 
                    (curr.timestamp == prev.timestamp and curr.agg_trade_id <= prev.agg_trade_id)):
                    raise ValueError(f"aggTrades data is not properly sorted at index {i}")
            
            logger.info(f"Loaded {len(aggtrades)} aggTrades from {file_path}")
            return aggtrades
            
        except Exception as e:
            if isinstance(e, (ValueError, RuntimeError)):
                raise
            raise RuntimeError(f"Failed to load aggTrades from {file_path}: {e}")
    
    async def fetch_and_load_aggtrades(self, 
                                      symbol: str,
                                      start_date: str,
                                      end_date: str) -> List[AggTrade]:
        """
        Fetch and load aggTrades data in one operation.
        
        Args:
            symbol: Trading symbol (e.g., 'BTCUSDT')
            start_date: Start date in YYYY-MM-DD format
            end_date: End date in YYYY-MM-DD format
            
        Returns:
            List of AggTrade objects sorted by timestamp
            
        Raises:
            ValueError: If parameters are invalid
            RuntimeError: If fetch/load fails
        """
        # Fetch data files
        file_paths = await self.fetch_aggtrades(symbol, start_date, end_date)
        
        # Load all trades from all files
        all_trades = []
        for file_path in file_paths:
            trades = self.load_aggtrades_from_file(file_path)
            all_trades.extend(trades)
        
        # Sort by timestamp, then by agg_trade_id for consistency
        all_trades.sort(key=lambda t: (t.timestamp, t.agg_trade_id))
        
        logger.info(f"Loaded total of {len(all_trades)} aggTrades for {symbol}")
        return all_trades
    
    def cleanup_data_directory(self, 
                              symbol: Optional[str] = None,
                              older_than_days: int = 30) -> None:
        """
        Clean up old downloaded data files.
        
        Args:
            symbol: Specific symbol to clean (None for all symbols)
            older_than_days: Remove files older than this many days
        """
        if not self.data_directory.exists():
            return
            
        import time
        cutoff_time = time.time() - (older_than_days * 24 * 60 * 60)
        
        pattern = f"*{symbol}*" if symbol else "*"
        removed_count = 0
        
        for file_path in self.data_directory.rglob(pattern):
            if file_path.is_file() and file_path.stat().st_mtime < cutoff_time:
                file_path.unlink()
                removed_count += 1
                
        logger.info(f"Cleaned up {removed_count} old data files")


# Convenience function for direct usage
async def fetch_um_futures_aggtrades(symbol: str,
                                    start_date: str,
                                    end_date: str,
                                    data_directory: Optional[Path] = None) -> List[AggTrade]:
    """
    Convenience function to fetch UM Futures aggTrades data.
    
    Args:
        symbol: Trading symbol (e.g., 'BTCUSDT')
        start_date: Start date in YYYY-MM-DD format
        end_date: End date in YYYY-MM-DD format
        data_directory: Optional data directory path
        
    Returns:
        List of AggTrade objects sorted by timestamp
    """
    fetcher = UMFuturesDataFetcher(data_directory=data_directory)
    return await fetcher.fetch_and_load_aggtrades(symbol, start_date, end_date)