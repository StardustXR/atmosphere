use rustc_hash::FxHashMap;
use stardust_xr_fusion::{
	node::NodeType,
	spatial::{Spatial, SpatialAspect, SpatialRef, Zone, ZoneAspect, ZoneHandler},
};

pub struct AutoZoneCapture(pub Zone, pub FxHashMap<u64, Spatial>);
impl ZoneHandler for AutoZoneCapture {
	fn enter(&mut self, spatial: SpatialRef) {
		self.0.capture(&spatial).unwrap();
	}
	fn capture(&mut self, spatial: Spatial) {
		spatial.set_spatial_parent_in_place(&self.0).unwrap();
		self.1.insert(spatial.node().get_id().unwrap(), spatial);
	}
	fn release(&mut self, _id: u64) {}
	fn leave(&mut self, _id: u64) {}
}
