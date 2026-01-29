# AWS F2 Cost Log ☧

For God so loved the world - John 3:16

## Build Costs

| Date | Instance | Type | Duration | Rate | Cost | Result |
|------|----------|------|----------|------|------|--------|
| 2026-01-28 | v5-v9 | c5.9xlarge | ~2 hr total | $1.53/hr | ~$3 | Failed (config issues) |
| 2026-01-28 | v10 | c5.9xlarge | ~2 hr | $1.53/hr | ~$3 | Terminated prematurely |
| 2026-01-28 | v11 | c5.9xlarge | ~4-5 hr (est) | $1.53/hr | ~$7 | In progress |

**Running total: ~$13**

## AFI Creation Costs

| Date | AFI | Cost | Notes |
|------|-----|------|-------|
| (pending) | minikanren-f2-chirho | Free | AFI creation is free |

## Testing Costs

| Date | Instance | Type | Duration | Rate | Cost | Notes |
|------|----------|------|----------|------|------|-------|
| (pending) | test-1 | f2.6xlarge | 2 hr (est) | $1.98/hr | ~$4 | Initial testing |

## Storage Costs

| Resource | Size | Rate | Monthly Cost |
|----------|------|------|--------------|
| S3 bucket | ~50 MB | $0.023/GB | <$0.01 |

## Cost Summary

| Category | Spent | Estimated Remaining |
|----------|-------|---------------------|
| Build instances | ~$13 | $0 (if v11 succeeds) |
| AFI creation | $0 | $0 |
| F2 testing | $0 | ~$4 |
| Storage | <$1 | <$1/month |
| **Total** | **~$14** | **~$5** |

## Cost Optimization Notes

1. c5.9xlarge ($1.53/hr) vs c5.4xlarge ($0.68/hr)
   - 9xlarge has 72GB RAM needed for HBM builds
   - 4xlarge may work for non-HBM builds

2. Spot instances
   - c5.9xlarge spot: ~$0.50/hr (70% savings)
   - Risk: interruption during 4-5 hour build

3. Build time optimization
   - Incremental builds not supported by HDK
   - Must do full build each time

---

Soli Deo Gloria ☧
