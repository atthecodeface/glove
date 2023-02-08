LOCATE_CAMERA = ./target/release/locate_camera
CREATE_RAYS = ./target/release/create_rays
COMBINE_RAYS = ./target/release/combine_rays

CAMERA_DB = --db camera_db.json
NPS_JSON = nps_noughts_and_crosses.json
NPS = --nps ${NPS_JSON}
IMAGE_DIR = /Users/gavinjstark/Git/Images
LOCATE_STEPS = --steps 31
LOCATE_SIZES = 100 50 10 5 1

IMAGE = 4v3a6042

.PHONY: build
build:
	cargo build --release

.PHONY: test
test:
	cargo test --release

.PHONY: locate
locate: located_${IMAGE}.json

located_${IMAGE}.json: ${LOCATE_CAMERA} pms_nac_${IMAGE}.json camera_nac_${IMAGE}.json ${NPS_JSON}
	${LOCATE_CAMERA} ${CAMERA_DB} ${NPS} --total ${LOCATE_STEPS} ${LOCATE_SIZES} --pms pms_nac_${IMAGE}.json --camera camera_nac_${IMAGE}.json --read ${IMAGE_DIR}/${IMAGE}.JPG --write a.png > $@

.PHONY: create_rays
create_rays: ${IMAGE}_rays.json

.PHONY: ${IMAGE}_rays.json
${IMAGE}_rays.json: ${CREATE_RAYS}
	${CREATE_RAYS} ${CAMERA_DB} ${NPS} --pms pms_nac_${IMAGE}.json --camera located_${IMAGE}.json > $@

.PHONY: combine_rays
combine_rays:
	${COMBINE_RAYS} ${NPS} 4v3a6040_rays.json 4v3a6041_rays.json 4v3a6042_rays.json

.PHONY: rays_from_model
rays_from_model: ${IMAGE}_rays_from_model.json

.PHONY: ${IMAGE}_rays_from_model.json
${IMAGE}_rays_from_model.json: ${CREATE_RAYS}
	${CREATE_RAYS} ${CAMERA_DB} ${NPS} --from_model --pms pms_nac_${IMAGE}.json --camera located_${IMAGE}.json > $@

.PHONY: blah
blah:
	# $(MAKE) build
	$(MAKE) IMAGE=4v3a6040_rays locate
	$(MAKE) IMAGE=4v3a6041 locate
	$(MAKE) IMAGE=4v3a6042 locate
	$(MAKE) IMAGE=4v3a6040 create_rays
	$(MAKE) IMAGE=4v3a6041 create_rays
	$(MAKE) IMAGE=4v3a6042 create_rays
	$(MAKE) IMAGE=4v3a6040 rays_from_model
	$(MAKE) IMAGE=4v3a6041 rays_from_model
	$(MAKE) IMAGE=4v3a6042 rays_from_model
	$(MAKE) combine_rays

