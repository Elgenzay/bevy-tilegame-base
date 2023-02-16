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

	pub fn get_outline_index(self) -> usize {
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
								} else {
									if self.bottom_right {
										18
									} else {
										19
									}
								}
							} else {
								if self.bottom_left {
									if self.bottom_right {
										25
									} else {
										27
									}
								} else {
									if self.bottom_right {
										39
									} else {
										42
									}
								}
							}
						} else {
							if self.top_right {
								if self.bottom_left {
									if self.bottom_right {
										26
									} else {
										47
									}
								} else {
									if self.bottom_right {
										28
									} else {
										41
									}
								}
							} else {
								if self.bottom_left {
									if self.bottom_right {
										20
									} else {
										34
									}
								} else {
									if self.bottom_right {
										33
									} else {
										23
									}
								}
							}
						}
					} else {
						if self.top_left {
							if self.top_right {
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
							} else {
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
							}
						} else {
							if self.top_right {
								if self.bottom_left {
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
							} else {
								if self.bottom_left {
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
							}
						}
					}
				} else {
					//////
					// ?0?
					// .0?
					// ?0?
					if self.right {
						// ?0?
						// .00
						// ?0?
						if self.top_left {
							// 00?
							// .00
							// ?0?
							if self.top_right {
								// 000
								// .00
								// ?0?
								if self.bottom_left {
									// 000
									// .00
									// 00?
									if self.bottom_right {
										21
									} else {
										43
									}
								} else {
									// 000
									// .00
									// .0?
									if self.bottom_right {
										21
									} else {
										43
									}
								}
							} else {
								// 00.
								// .00
								// ?0?
								if self.bottom_left {
									if self.bottom_right {
										35
									} else {
										12
									}
								} else {
									if self.bottom_right {
										35
									} else {
										12
									}
								}
							}
						} else {
							// .0?
							// .00
							// ?0?
							if self.top_right {
								// .00
								// .00
								// ?0?
								if self.bottom_left {
									if self.bottom_right {
										21
									} else {
										43
									}
								} else {
									if self.bottom_right {
										21
									} else {
										43
									}
								}
							} else {
								// .0.
								// .00
								// ?0?
								if self.bottom_left {
									if self.bottom_right {
										35
									} else {
										12
									}
								} else {
									if self.bottom_right {
										35
									} else {
										12
									}
								}
							}
						}
					} else {
						// ?0?
						// .0.
						// ?0?

						if self.top_left {
							// 00?
							// .0.
							// ?0?
							if self.top_right {
								// 000
								// .0.
								// ?0?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							} else {
								// 00.
								// .0.
								// ?0?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							}
						} else {
							// .0?
							// .0.
							// ?0?
							if self.top_right {
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							} else {
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							}
						}
						11
					}
				}
			} else {
				// ?0?
				// ?0?
				// ?.?
				if self.left {
					// ?0?
					// 00?
					// ?.?
					if self.right {
						// ?0?
						// 000
						// ?.?
						if self.top_left {
							// 00?
							// 000
							// ?.?
							if self.top_right {
								// 000
								// 000
								// ?.?
								if self.bottom_left {
									// 000
									// 000
									// 0.?
									if self.bottom_right {
									} else {
									}
								} else {
									// 000
									// 000
									// ..?
									if self.bottom_right {
									} else {
									}
								}
								30
							} else {
								// 00.
								// 000
								// ?.?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
								46
							}
						} else {
							// .0?
							// 000
							// ?.?
							if self.top_right {
								// .00
								// 000
								// ?.?
								if self.bottom_left {
									// .00
									// 000
									// 0.?
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
								45
							} else {
								// .0.
								// 000
								// ?.?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
								5
							}
						}
					} else {
						// ?0?
						// 00.
						// ?.?
						if self.top_left {
							// 00?
							// 00.
							// ?.?

							if self.top_right {
								// 000
								// 00.
								// ?.?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							} else {
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							}
							32
						} else {
							// .0?
							// 00.
							// ?.?
							if self.top_right {
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							} else {
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							}
							10
						}
					}
				} else {
					//////
					// ?0?
					// .0?
					// ?.?
					if self.right {
						// ?0?
						// .00
						// ?.?
						if self.top_left {
							// 00?
							// .00
							// ?.?
							if self.top_right {
								// 000
								// .00
								// ?.?
								if self.bottom_left {
									// 000
									// .00
									// 0.?
									if self.bottom_right {
									} else {
									}
								} else {
									// 000
									// .00
									// ..?
									if self.bottom_right {
									} else {
									}
								}
								16
							} else {
								// 00.
								// .00
								// ?.?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
								9
							}
						} else {
							// .0?
							// .00
							// ?.?
							if self.top_right {
								// .00
								// .00
								// ?.?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
								16
							} else {
								// .0.
								// .00
								// ?.?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
								9
							}
						}
					} else {
						// ?0?
						// .0.
						// ?.?

						if self.top_left {
							// 00?
							// .0.
							// ?.?
							if self.top_right {
								// 000
								// .0.
								// ?.?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							} else {
								// 00.
								// .0.
								// ?.?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							}
						} else {
							// .0?
							// .0.
							// ?.?
							if self.top_right {
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							} else {
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							}
						}
						7
					}
				}
			}
		} else {
			////////////////////////////////////////////////////////////////////////////////////////////
			//halfway point
			if self.bottom {
				// ?.?
				// ?0?
				// ?0?
				if self.left {
					// ?.?
					// 00?
					// ?0?
					if self.right {
						// ?.?
						// 000
						// ?0?
						if self.top_left {
							// 0.?
							// 000
							// ?0?
							if self.top_right {
								// 0.0
								// 000
								// ?0?
								if self.bottom_left {
									// 0.0
									// 000
									// 00?
									if self.bottom_right {
										29
									} else {
										38
									}
								} else {
									// 0.0
									// 000
									// .0?
									if self.bottom_right {
										37
									} else {
										4
									}
								}
							} else {
								// 0..
								// 000
								// ?0?
								if self.bottom_left {
									if self.bottom_right {
										29
									} else {
										38
									}
								} else {
									if self.bottom_right {
										37
									} else {
										4
									}
								}
							}
						} else {
							// ..?
							// 000
							// ?0?
							if self.top_right {
								if self.bottom_left {
									if self.bottom_right {
										29
									} else {
										38
									}
								} else {
									if self.bottom_right {
										37
									} else {
										4
									}
								}
							} else {
								if self.bottom_left {
									if self.bottom_right {
										29
									} else {
										38
									}
								} else {
									if self.bottom_right {
										37
									} else {
										4
									}
								}
							}
						}
					} else {
						// ?.?
						// 00.
						// ?0?
						if self.top_left {
							// 0.?
							// 00.
							// ?0?
							if self.top_right {
								// 0.0
								// 00.
								// ?0?
								if self.bottom_left {
									// 0.0
									// 00.
									// 00?
									if self.bottom_right {
									} else {
									}
									24
								} else {
									// 0.0
									// 00.
									// .0?
									if self.bottom_right {
									} else {
									}
									2
								}
							} else {
								// 0..
								// 00.
								// ?0?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
									24
								} else {
									if self.bottom_right {
									} else {
									}
									2
								}
							}
						} else {
							// ..?
							// 00.
							// ?0?
							if self.top_right {
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
									24
								} else {
									if self.bottom_right {
									} else {
									}
									2
								}
							} else {
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
									24
								} else {
									if self.bottom_right {
									} else {
									}
									2
								}
							}
						}
					}
				} else {
					// ?.?
					// .0?
					// ?0?
					if self.right {
						// ?.?
						// .00
						// ?0?
						if self.top_left {
							// 0.?
							// .00
							// ?0?
							if self.top_right {
								// 0.0
								// .00
								// ?0?
								if self.bottom_left {
									// 0.0
									// .00
									// 00?
									if self.bottom_right {
										8
									} else {
										1
									}
								} else {
									// 0.0
									// .00
									// .0?
									if self.bottom_right {
										8
									} else {
										1
									}
								}
							} else {
								// 0..
								// .00
								// ?0?
								if self.bottom_left {
									if self.bottom_right {
										8
									} else {
										1
									}
								} else {
									if self.bottom_right {
										8
									} else {
										1
									}
								}
							}
						} else {
							// ..?
							// .00
							// ?0?
							if self.top_right {
								// ..0
								// .00
								// ?0?
								if self.bottom_left {
									if self.bottom_right {
										8
									} else {
										1
									}
								} else {
									if self.bottom_right {
										8
									} else {
										1
									}
								}
							} else {
								// ...
								// .00
								// ?0?
								if self.bottom_left {
									if self.bottom_right {
										8
									} else {
										1
									}
								} else {
									if self.bottom_right {
										8
									} else {
										1
									}
								}
							}
						}
					} else {
						// ?.?
						// .0.
						// ?0?

						if self.top_left {
							// 0.?
							// .0.
							// ?0?
							if self.top_right {
								// 0.0
								// .0.
								// ?0?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							} else {
								// 0..
								// .0.
								// ?0?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							}
						} else {
							// ..?
							// .0.
							// ?0?
							if self.top_right {
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							} else {
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							}
						}
						14
					}
				}
			} else {
				// ?.?
				// ?0?
				// ?.?
				if self.left {
					// ?.?
					// 00?
					// ?.?
					if self.right {
						// ?.?
						// 000
						// ?.?
						if self.top_left {
							// 0.?
							// 000
							// ?.?
							if self.top_right {
								// 0.0
								// 000
								// ?.?
								if self.bottom_left {
									// 0.0
									// 000
									// 0.?
									if self.bottom_right {
									} else {
									}
								} else {
									// 0.0
									// 000
									// ..?
									if self.bottom_right {
									} else {
									}
								}
							} else {
								// 0..
								// 000
								// ?.?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							}
						} else {
							// ..?
							// 000
							// ?.?
							if self.top_right {
								// ..0
								// 000
								// ?.?
								if self.bottom_left {
									// ..0
									// 000
									// 0.?
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							} else {
								// ...
								// 000
								// ?.?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							}
						}
						3
					} else {
						// ?.?
						// 00.
						// ?.?
						if self.top_left {
							// 0.?
							// 00.
							// ?.?

							if self.top_right {
								// 0.0
								// 00.
								// ?.?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							} else {
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							}
						} else {
							// ..?
							// 00.
							// ?.?
							if self.top_right {
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							} else {
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							}
						}
						6
					}
				} else {
					//////
					// ?.?
					// .0?
					// ?.?
					if self.right {
						// ?.?
						// .00
						// ?.?
						if self.top_left {
							// 0.?
							// .00
							// ?.?
							if self.top_right {
								// 0.0
								// .00
								// ?.?
								if self.bottom_left {
									// 0.0
									// .00
									// 0.?
									if self.bottom_right {
									} else {
									}
								} else {
									// 0.0
									// .00
									// ..?
									if self.bottom_right {
									} else {
									}
								}
							} else {
								// 0..
								// .00
								// ?.?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							}
						} else {
							// ..?
							// .00
							// ?.?
							if self.top_right {
								// ..0
								// .00
								// ?.?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							} else {
								// ...
								// .00
								// ?.?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							}
						}
						15
					} else {
						// ?.?
						// .0.
						// ?.?

						if self.top_left {
							// 0.?
							// .0.
							// ?.?
							if self.top_right {
								// 0.0
								// .0.
								// ?.?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							} else {
								// 0..
								// .0.
								// ?.?
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							}
						} else {
							// ..?
							// .0.
							// ?.?
							if self.top_right {
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							} else {
								if self.bottom_left {
									if self.bottom_right {
									} else {
									}
								} else {
									if self.bottom_right {
									} else {
									}
								}
							}
						}
						31
					}
				}
			}
		}
	}
}
