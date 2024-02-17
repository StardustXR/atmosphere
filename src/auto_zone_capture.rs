use rustc_hash::FxHashMap;
use stardust_xr_fusion::spatial::{Spatial, SpatialAspect, Zone, ZoneAspect, ZoneHandler};

pub struct AutoZoneCapture(pub Zone, pub FxHashMap<String, Spatial>);
impl ZoneHandler for AutoZoneCapture {
	fn enter(&mut self, uid: String, spatial: Spatial) {
		self.0.capture(&spatial).unwrap();
		self.1.insert(uid, spatial);
	}
	fn capture(&mut self, uid: String) {
		self.1
			.get(&uid)
			.unwrap()
			.set_spatial_parent_in_place(&self.0)
			.unwrap();
	}
	fn release(&mut self, _uid: String) {}
	fn leave(&mut self, _uid: String) {}
}
