import sys
from pathlib import Path
import arex as a
base=Path(__file__).resolve().parents[1]
for name,schema in [('release-plan','PlanInputV1'),('release-parse','ParseInputV1'),('overview-plan','NavigationContextV1'),('overview-parse','NavigationParseInputV1'),('episode-plan','NavigationContextV1'),('episode-parse','NavigationParseInputV1')]:
    a.validate(a.strict((base/'fixtures/wire'/f'{name}-input.json').read_bytes(),4194304),schema)
    output=base/'build/wire'/f'{name}-output.json'
    if output.exists():
        target={'release-plan':'PlanOutputV1','release-parse':'ParseOutputV1','overview-plan':'NavigationPlanOutputV1','episode-plan':'NavigationPlanOutputV1','overview-parse':'NavigationParseOutputV1','episode-parse':'NavigationParseOutputV1'}[name]
        a.validate(a.strict(output.read_bytes(),1048576),target)
print('Shared v1 wire schema fixtures passed')
