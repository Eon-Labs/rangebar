"""
Parquet converter with schema validation for range bar data.

Converts aggTrades and range bar data to efficient Parquet format with
proper schema validation and compression for high-performance processing.
"""

import logging
from pathlib import Path
from typing import List, Dict, Any, Optional, Union
from datetime import datetime

import pyarrow as pa
import pyarrow.parquet as pq
import pyarrow.compute as pc

from .range_bars import AggTrade


logger = logging.getLogger(__name__)


class ParquetConverter:
    """
    Converts aggTrades and range bar data to Parquet format with schema validation.
    
    Provides efficient storage and retrieval for large-scale range bar processing.
    """
    
    # Schema for aggTrades data
    AGGTRADES_SCHEMA = pa.schema([
        ('agg_trade_id', pa.int64()),
        ('price', pa.float64()),  # Use float64 for better compatibility
        ('quantity', pa.float64()),
        ('first_trade_id', pa.int64()),
        ('last_trade_id', pa.int64()),
        ('timestamp', pa.timestamp('ms')),  # Millisecond precision
        ('is_buyer_maker', pa.bool_()),
        ('symbol', pa.string()),  # Add symbol for multi-symbol datasets
    ])
    
    # Schema for range bar data
    RANGE_BARS_SCHEMA = pa.schema([
        ('open_time', pa.timestamp('ms')),
        ('close_time', pa.timestamp('ms')),
        ('open', pa.float64()),
        ('high', pa.float64()),
        ('low', pa.float64()),
        ('close', pa.float64()),
        ('volume', pa.float64()),
        ('turnover', pa.float64()),
        ('trade_count', pa.int32()),
        ('first_id', pa.int64()),
        ('last_id', pa.int64()),
        ('symbol', pa.string()),
        ('threshold_bps', pa.int32()),  # Threshold used for this bar
    ])
    
    def __init__(self, compression: str = "snappy"):
        """
        Initialize the Parquet converter.
        
        Args:
            compression: Compression algorithm ("snappy", "gzip", "lz4", "brotli")
        """
        valid_compressions = ["snappy", "gzip", "lz4", "brotli"]
        if compression not in valid_compressions:
            raise ValueError(f"Compression must be one of {valid_compressions}, got: {compression}")
            
        self.compression = compression
        logger.info(f"Initialized ParquetConverter with {compression} compression")
    
    def validate_aggtrades_data(self, aggtrades: List[AggTrade]) -> None:
        """
        Validate aggTrades data before conversion.
        
        Args:
            aggtrades: List of AggTrade objects
            
        Raises:
            ValueError: If data validation fails
        """
        if not aggtrades:
            raise ValueError("aggTrades list cannot be empty")
        
        # Check data consistency
        for i, trade in enumerate(aggtrades):
            if not isinstance(trade, AggTrade):
                raise ValueError(f"Item at index {i} is not an AggTrade object")
            
            # Validate required fields
            if trade.agg_trade_id <= 0:
                raise ValueError(f"Invalid agg_trade_id at index {i}: {trade.agg_trade_id}")
            
            if trade.price <= 0:
                raise ValueError(f"Invalid price at index {i}: {trade.price}")
                
            if trade.quantity <= 0:
                raise ValueError(f"Invalid quantity at index {i}: {trade.quantity}")
                
            if trade.timestamp <= 0:
                raise ValueError(f"Invalid timestamp at index {i}: {trade.timestamp}")
        
        # Validate sorting
        for i in range(1, len(aggtrades)):
            prev = aggtrades[i-1]
            curr = aggtrades[i]
            
            if (curr.timestamp < prev.timestamp or 
                (curr.timestamp == prev.timestamp and curr.agg_trade_id <= prev.agg_trade_id)):
                raise ValueError(f"aggTrades data is not properly sorted at index {i}")
        
        logger.info(f"Validated {len(aggtrades)} aggTrades")
    
    def aggtrades_to_parquet(self, 
                           aggtrades: List[AggTrade],
                           output_path: Path,
                           symbol: str) -> None:
        """
        Convert aggTrades to Parquet format.
        
        Args:
            aggtrades: List of AggTrade objects
            output_path: Path to save Parquet file
            symbol: Trading symbol for the data
            
        Raises:
            ValueError: If data validation fails
            RuntimeError: If conversion fails
        """
        # Validate input data
        self.validate_aggtrades_data(aggtrades)
        
        if not symbol or not isinstance(symbol, str):
            raise ValueError("Symbol must be a non-empty string")
        
        try:
            logger.info(f"Converting {len(aggtrades)} aggTrades to Parquet: {output_path}")
            
            # Extract data into arrays
            agg_trade_ids = [trade.agg_trade_id for trade in aggtrades]
            prices = [float(str(trade.price)) for trade in aggtrades]
            quantities = [float(str(trade.quantity)) for trade in aggtrades]
            first_trade_ids = [trade.first_trade_id for trade in aggtrades]
            last_trade_ids = [trade.last_trade_id for trade in aggtrades]
            timestamps = [trade.timestamp for trade in aggtrades]
            is_buyer_makers = [trade.is_buyer_maker for trade in aggtrades]
            symbols = [symbol] * len(aggtrades)
            
            # Create PyArrow table
            table = pa.table({
                'agg_trade_id': agg_trade_ids,
                'price': prices,
                'quantity': quantities,
                'first_trade_id': first_trade_ids,
                'last_trade_id': last_trade_ids,
                'timestamp': pa.array(timestamps, type=pa.timestamp('ms')),
                'is_buyer_maker': is_buyer_makers,
                'symbol': symbols,
            }, schema=self.AGGTRADES_SCHEMA)
            
            # Ensure output directory exists
            output_path.parent.mkdir(parents=True, exist_ok=True)
            
            # Write to Parquet
            pq.write_table(
                table, 
                output_path,
                compression=self.compression,
                use_dictionary=True,  # Enable dictionary encoding for strings
                row_group_size=100000,  # Optimize for batch processing
            )
            
            file_size = output_path.stat().st_size
            logger.info(f"Successfully wrote {len(aggtrades)} aggTrades to {output_path} ({file_size:,} bytes)")
            
        except Exception as e:
            if isinstance(e, ValueError):
                raise
            raise RuntimeError(f"Failed to convert aggTrades to Parquet: {e}")
    
    def range_bars_to_parquet(self,
                             range_bars: List[Dict[str, Any]],
                             output_path: Path,
                             symbol: str,
                             threshold_bps: int) -> None:
        """
        Convert range bars to Parquet format.
        
        Args:
            range_bars: List of range bar dictionaries
            output_path: Path to save Parquet file
            symbol: Trading symbol for the data
            threshold_bps: Threshold in basis points used for range bars
            
        Raises:
            ValueError: If data validation fails
            RuntimeError: If conversion fails
        """
        if not range_bars:
            raise ValueError("Range bars list cannot be empty")
        
        if not symbol or not isinstance(symbol, str):
            raise ValueError("Symbol must be a non-empty string")
            
        if threshold_bps <= 0:
            raise ValueError("Threshold BPS must be positive")
        
        try:
            logger.info(f"Converting {len(range_bars)} range bars to Parquet: {output_path}")
            
            # Extract data into arrays
            open_times = [int(bar['open_time']) for bar in range_bars]
            close_times = [int(bar['close_time']) for bar in range_bars]
            opens = [float(str(bar['open'])) for bar in range_bars]
            highs = [float(str(bar['high'])) for bar in range_bars]
            lows = [float(str(bar['low'])) for bar in range_bars]
            closes = [float(str(bar['close'])) for bar in range_bars]
            volumes = [float(str(bar['volume'])) for bar in range_bars]
            turnovers = [float(str(bar['turnover'])) for bar in range_bars]
            trade_counts = [int(bar['trade_count']) for bar in range_bars]
            first_ids = [int(bar['first_id']) for bar in range_bars]
            last_ids = [int(bar['last_id']) for bar in range_bars]
            symbols = [symbol] * len(range_bars)
            threshold_bps_list = [threshold_bps] * len(range_bars)
            
            # Create PyArrow table
            table = pa.table({
                'open_time': pa.array(open_times, type=pa.timestamp('ms')),
                'close_time': pa.array(close_times, type=pa.timestamp('ms')),
                'open': opens,
                'high': highs,
                'low': lows,
                'close': closes,
                'volume': volumes,
                'turnover': turnovers,
                'trade_count': trade_counts,
                'first_id': first_ids,
                'last_id': last_ids,
                'symbol': symbols,
                'threshold_bps': threshold_bps_list,
            }, schema=self.RANGE_BARS_SCHEMA)
            
            # Ensure output directory exists
            output_path.parent.mkdir(parents=True, exist_ok=True)
            
            # Write to Parquet
            pq.write_table(
                table,
                output_path,
                compression=self.compression,
                use_dictionary=True,
                row_group_size=10000,  # Smaller row groups for range bars
            )
            
            file_size = output_path.stat().st_size
            logger.info(f"Successfully wrote {len(range_bars)} range bars to {output_path} ({file_size:,} bytes)")
            
        except Exception as e:
            if isinstance(e, ValueError):
                raise
            raise RuntimeError(f"Failed to convert range bars to Parquet: {e}")
    
    def load_aggtrades_from_parquet(self, 
                                   parquet_path: Path,
                                   symbol_filter: Optional[str] = None,
                                   start_time: Optional[int] = None,
                                   end_time: Optional[int] = None) -> pa.Table:
        """
        Load aggTrades from Parquet file with optional filtering.
        
        Args:
            parquet_path: Path to Parquet file
            symbol_filter: Optional symbol to filter by
            start_time: Optional start timestamp (milliseconds)
            end_time: Optional end timestamp (milliseconds)
            
        Returns:
            PyArrow table with filtered data
            
        Raises:
            RuntimeError: If file cannot be read
        """
        if not parquet_path.exists():
            raise RuntimeError(f"Parquet file does not exist: {parquet_path}")
        
        try:
            logger.info(f"Loading aggTrades from Parquet: {parquet_path}")
            
            # Build filter conditions
            filters = []
            
            if symbol_filter:
                filters.append(('symbol', '=', symbol_filter))
            
            if start_time is not None:
                # Convert to timestamp
                start_ts = pa.scalar(start_time, type=pa.timestamp('ms'))
                filters.append(('timestamp', '>=', start_ts))
                
            if end_time is not None:
                # Convert to timestamp  
                end_ts = pa.scalar(end_time, type=pa.timestamp('ms'))
                filters.append(('timestamp', '<=', end_ts))
            
            # Load with filters (only if filters exist)
            if filters:
                table = pq.read_table(parquet_path, filters=filters)
            else:
                table = pq.read_table(parquet_path)
            
            logger.info(f"Loaded {table.num_rows} aggTrades from Parquet")
            return table
            
        except Exception as e:
            raise RuntimeError(f"Failed to load aggTrades from Parquet: {e}")
    
    def load_range_bars_from_parquet(self,
                                   parquet_path: Path,
                                   symbol_filter: Optional[str] = None,
                                   threshold_filter: Optional[int] = None) -> pa.Table:
        """
        Load range bars from Parquet file with optional filtering.
        
        Args:
            parquet_path: Path to Parquet file
            symbol_filter: Optional symbol to filter by
            threshold_filter: Optional threshold BPS to filter by
            
        Returns:
            PyArrow table with filtered data
            
        Raises:
            RuntimeError: If file cannot be read
        """
        if not parquet_path.exists():
            raise RuntimeError(f"Parquet file does not exist: {parquet_path}")
        
        try:
            logger.info(f"Loading range bars from Parquet: {parquet_path}")
            
            # Build filter conditions
            filters = []
            
            if symbol_filter:
                filters.append(('symbol', '=', symbol_filter))
            
            if threshold_filter is not None:
                filters.append(('threshold_bps', '=', threshold_filter))
            
            # Load with filters (only if filters exist)
            if filters:
                table = pq.read_table(parquet_path, filters=filters)
            else:
                table = pq.read_table(parquet_path)
            
            logger.info(f"Loaded {table.num_rows} range bars from Parquet")
            return table
            
        except Exception as e:
            raise RuntimeError(f"Failed to load range bars from Parquet: {e}")
    
    def get_file_info(self, parquet_path: Path) -> Dict[str, Any]:
        """
        Get metadata information about a Parquet file.
        
        Args:
            parquet_path: Path to Parquet file
            
        Returns:
            Dictionary with file metadata
            
        Raises:
            RuntimeError: If file cannot be read
        """
        if not parquet_path.exists():
            raise RuntimeError(f"Parquet file does not exist: {parquet_path}")
        
        try:
            parquet_file = pq.ParquetFile(parquet_path)
            metadata = parquet_file.metadata
            
            info = {
                'num_rows': metadata.num_rows,
                'num_columns': metadata.num_columns,
                'num_row_groups': metadata.num_row_groups,
                'serialized_size': metadata.serialized_size,
                'compression': str(metadata.row_group(0).column(0).compression) if metadata.num_row_groups > 0 else None,
                'schema': str(parquet_file.schema_arrow),
                'file_size': parquet_path.stat().st_size,
            }
            
            return info
            
        except Exception as e:
            raise RuntimeError(f"Failed to get Parquet file info: {e}")


# Convenience functions
def aggtrades_to_parquet(aggtrades: List[AggTrade],
                        output_path: Path,
                        symbol: str,
                        compression: str = "snappy") -> None:
    """
    Convenience function to convert aggTrades to Parquet.
    """
    converter = ParquetConverter(compression=compression)
    converter.aggtrades_to_parquet(aggtrades, output_path, symbol)


def range_bars_to_parquet(range_bars: List[Dict[str, Any]],
                         output_path: Path,
                         symbol: str,
                         threshold_bps: int,
                         compression: str = "snappy") -> None:
    """
    Convenience function to convert range bars to Parquet.
    """
    converter = ParquetConverter(compression=compression)
    converter.range_bars_to_parquet(range_bars, output_path, symbol, threshold_bps)