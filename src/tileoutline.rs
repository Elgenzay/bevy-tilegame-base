pub struct ConnectedNeighbors {
	pub top_left: bool,
	pub top: bool,
	pub top_right: bool,
	pub left: bool,
	pub right: bool,
	pub bottom: bool,
	pub bottom_left: bool,
	pub bottom_right: bool,
}

impl ConnectedNeighbors {
	pub fn new() -> Self {
		Self {
			top_left: false,
			top: false,
			top_right: false,
			left: false,
			right: false,
			bottom: false,
			bottom_left: false,
			bottom_right: false,
		}
	}

	#[allow(clippy::if_same_then_else)]
	pub fn get_outline_id(self) -> usize {
		if self.top {
			if self.bottom {
				if self.left {
					if self.right {
						if self.top_left {
							if self.top_right {
								if self.bottom_left {
									if self.bottom_right {
										40
									} else {
										17
									}
								} else if self.bottom_right {
									18
								} else {
									19
								}
							} else if self.bottom_left {
								if self.bottom_right {
									25
								} else {
									27
								}
							} else if self.bottom_right {
								39
							} else {
								42
							}
						} else if self.top_right {
							if self.bottom_left {
								if self.bottom_right {
									26
								} else {
									47
								}
							} else if self.bottom_right {
								28
							} else {
								41
							}
						} else if self.bottom_left {
							if self.bottom_right {
								20
							} else {
								34
							}
						} else if self.bottom_right {
							33
						} else {
							23
						}
					} else if self.top_left {
						if self.bottom_left {
							if self.bottom_right {
							} else {
							}
							22
						} else {
							if self.bottom_right {
							} else {
							}
							44
						}
					} else if self.bottom_left {
						if self.bottom_right {
						} else {
						}
						36
					} else {
						if self.bottom_right {
						} else {
						}
						13
					}
				} else if self.right {
					if self.top_right {
						if self.bottom_right {
							21
						} else {
							43
						}
					} else if self.bottom_right {
						35
					} else {
						12
					}
				} else {
					11
				}
			} else if self.left {
				if self.right {
					if self.top_left {
						if self.top_right {
							30
						} else {
							46
						}
					} else if self.top_right {
						45
					} else {
						5
					}
				} else if self.top_left {
					32
				} else {
					10
				}
			} else if self.right {
				if self.top_right {
					16
				} else {
					9
				}
			} else {
				7
			}
		} else if self.bottom {
			if self.left {
				if self.right {
					if self.bottom_left {
						if self.bottom_right {
							29
						} else {
							38
						}
					} else if self.bottom_right {
						37
					} else {
						4
					}
				} else if self.bottom_left {
					24
				} else {
					2
				}
			} else if self.right {
				if self.bottom_right {
					8
				} else {
					1
				}
			} else {
				14
			}
		} else if self.left {
			if self.right {
				3
			} else {
				6
			}
		} else if self.right {
			15
		} else {
			31
		}
	}
}
