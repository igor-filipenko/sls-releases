export type ReleaseFilter = {
  includeRc: boolean;
  includeMilestones: boolean;
};

export function loadReleaseFilter(): ReleaseFilter {
  try {
    const filter = localStorage.getItem("releaseFilter");
    return filter ? JSON.parse(filter) : { includeRc: false, includeMilestones: false };
  } catch (error) {
    console.error("Error loading release filter:", error);
    return { includeRc: false, includeMilestones: false };
  }
}

export function saveReleaseFilter(filter: ReleaseFilter) {
  localStorage.setItem("releaseFilter", JSON.stringify(filter));
}
