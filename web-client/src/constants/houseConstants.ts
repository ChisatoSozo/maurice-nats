import { feetAndInches } from "../utils/util";

export type SitePlan = {
  type: "sitePlan";
  width: number;
  depth: number;
  x: number;
  y: number;
};

export type Post = {
  type: "post";
  x: number;
  y: number;
};

export type HouseElement = SitePlan | Post;

export type Layer = {
  [element: string]: HouseElement;
};

export type HouseConstants = {
  [layer: string]: Layer;
};

export const houseConstants = {
  sitePlan: {
    property: {
      type: "sitePlan",
      width: feetAndInches(30, 0),
      depth: feetAndInches(80, 0),
      x: feetAndInches(0, 0),
      y: feetAndInches(0, 0),
    },
    house: {
      type: "sitePlan",
      width: feetAndInches(24, 0),
      depth: feetAndInches(32, 0),
      x: feetAndInches(0, 0),
      y: feetAndInches(6, 0),
    },
    driveway: {
      type: "sitePlan",
      width: feetAndInches(10, 0),
      depth: feetAndInches(18, 0),
      x: feetAndInches(5, 0),
      y: feetAndInches(31, 0),
    },
    shed: {
      type: "sitePlan",
      width: feetAndInches(8, 0),
      depth: feetAndInches(10, 0),
      x: feetAndInches(-8, 0),
      y: feetAndInches(-31, 0),
    },
  },
} as HouseConstants;
