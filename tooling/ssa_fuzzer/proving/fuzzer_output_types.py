from dataclasses import dataclass
from typing import Dict, List, Optional, Any, Union
import json


@dataclass
class Span:
    start: int
    end: int

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Span":
        return cls(start=data["start"], end=data["end"])


@dataclass
class Value:
    file: int
    span: Span

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Value":
        return cls(file=data["file"], span=Span.from_dict(data["span"]))


@dataclass
class Location:
    parent: Optional[int]
    value: Value

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Location":
        return cls(parent=data["parent"], value=Value.from_dict(data["value"]))


@dataclass
class LocationTree:
    locations: List[Location]

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "LocationTree":
        return cls(locations=[Location.from_dict(loc) for loc in data["locations"]])


@dataclass
class Debug:
    acir_locations: Dict[str, Any]
    brillig_locations: Dict[str, Any]
    brillig_procedure_locs: Dict[str, Any]
    functions: Dict[str, Any]
    location_tree: LocationTree
    types: Dict[str, Any]
    variables: Dict[str, Any]

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Debug":
        return cls(
            acir_locations=data["acir_locations"],
            brillig_locations=data["brillig_locations"],
            brillig_procedure_locs=data["brillig_procedure_locs"],
            functions=data["functions"],
            location_tree=LocationTree.from_dict(data["location_tree"]),
            types=data["types"],
            variables=data["variables"],
        )


@dataclass
class ABI:
    error_types: Dict[str, Any]
    parameters: List[Any]
    return_type: Optional[Any]

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ABI":
        return cls(
            error_types=data["error_types"],
            parameters=data["parameters"],
            return_type=data["return_type"],
        )


@dataclass
class Program:
    abi: ABI
    brillig_names: List[str]
    bytecode: str
    debug: List[Debug]
    file_map: Dict[str, Any]
    hash: int
    names: List[str]
    noir_version: str
    warnings: List[Any]

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Program":
        return cls(
            abi=ABI.from_dict(data["abi"]),
            brillig_names=data["brillig_names"],
            bytecode=data["bytecode"],
            debug=[Debug.from_dict(debug_item) for debug_item in data["debug"]],
            file_map=data["file_map"],
            hash=data["hash"],
            names=data["names"],
            noir_version=data["noir_version"],
            warnings=data["warnings"],
        )


@dataclass
class NoirProgramData:
    program: Program
    test_id: str
    witness_map_b64: str

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "NoirProgramData":
        return cls(
            program=Program.from_dict(data["program"]),
            test_id=data["test_id"],
            witness_map_b64=data["witness_map_b64"],
        )

    @classmethod
    def from_json(cls, json_str: str) -> "NoirProgramData":
        data = json.loads(json_str)
        return cls.from_dict(data)


def parse_noir_program_data(json_data: Union[str, Dict[str, Any]]) -> NoirProgramData:
    if isinstance(json_data, str):
        return NoirProgramData.from_json(json_data)
    elif isinstance(json_data, dict):
        return NoirProgramData.from_dict(json_data)
    else:
        raise ValueError("Input must be either a JSON string or a dictionary")
