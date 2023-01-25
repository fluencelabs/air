#
#  Copyright 2023 Fluence Labs Limited
#
#  Licensed under the Apache License, Version 2.0 (the "License");
#  you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
#
"""Performance measurement database module."""

import datetime
import json
import logging
import os.path
import platform
from typing import Optional

from performance_metering.helpers import get_host_id, get_aquavm_version

DEFAULT_PATH = "benches/PERFORMANCE.json"
AQUAVM_TOML_PATH = "air/Cargo.toml"


class Db:
    """Performance measurement database."""

    path: str
    host_id: str
    data: hash

    def __init__(self, path: Optional[str], host_id=None):
        """Load data from file, if it exits."""
        if path is None:
            path = DEFAULT_PATH
        self.path = path

        if host_id is None:
            host_id = get_host_id()
        self.host_id = host_id

        try:
            with open(path, 'r') as inp:
                self.data = json.load(inp)
        except IOError as ex:
            logging.warn("cannot open data at %r: %s", path, ex)
            self.data = {}

    def record(self, bench, stats):
        """Record the bench stats."""
        if self.host_id not in self.data:
            self.data[self.host_id] = {"stats": {}}
        self.data[self.host_id]["stats"][bench.get_name()] = stats
        self.data[self.host_id]["platform"] = platform.platform()
        self.data[self.host_id]["datetime"] = str(
            datetime.datetime.now(datetime.timezone.utc)
        )
        self.data[self.host_id]["version"] = get_aquavm_version(
            AQUAVM_TOML_PATH
        )

    def save(self):
        """Save the database."""
        import tempfile

        with tempfile.NamedTemporaryFile(
                mode="w+",
                dir=os.path.dirname(self.path),
                prefix=os.path.basename(self.path),
                suffix='.tmpXXXXXX',
                delete=False,
        ) as out:
            try:
                json.dump(
                    self.data, out,
                    # for better diffs and readable files:
                    sort_keys=True,
                    indent=2,
                    ensure_ascii=False,
                )
                out.flush()
                os.rename(out.name, self.path)
            except Exception:
                os.remove(out.name)

    def __enter__(self):
        """Enter context manager."""
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        """Exit context manger, saving data if the exit is clean."""
        if exc_type is None:
            self.save()
