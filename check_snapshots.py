import json

with open('data/snapshots/system_snapshot.json') as f:
    data = json.load(f)

snapshots = data['snapshots']
print(f"📊 Toplam snapshot: {len(snapshots)}")
print(f"⏰ İlk timestamp: {snapshots[0]['timestamp']}")
print(f"⏰ Son timestamp: {snapshots[-1]['timestamp']}")
print(f"\n📈 Son 5 snapshot'ın CPU kullanımı:")
for i, snap in enumerate(snapshots[-5:], 1):
    print(f"  {i}. {snap['cpu_usage_pct']:.2f}% (timestamp: {snap['timestamp']})")
