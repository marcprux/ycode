// Copyright © 2026 ycode contributors
// SPDX-License-Identifier: MPL-2.0

//! Fixtures emitted by Apple's Swift encoder; see fixtures/README.md.
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{fmt::Debug, fs, path::Path};
use ycode_project::*;

fn roundtrip<T: DeserializeOwned + Serialize + PartialEq + Debug>(source: &str, path: &Path) {
    let original: T =
        json_five::from_str(source).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let encoded = serde_json::to_string_pretty(&original).unwrap();
    let decoded: T = serde_json::from_str(&encoded)
        .unwrap_or_else(|e| panic!("{}: {e}\n{encoded}", path.display()));
    assert_eq!(original, decoded, "{}", path.display());
    if let Some(directory) = std::env::var_os("YCODE_ROUNDTRIP_DIR") {
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            Path::new(&directory).join(path.file_name().unwrap()),
            encoded,
        )
        .unwrap();
    }
}

#[test]
fn decode_and_reencode_swift_fixtures() {
    let corpus: std::collections::BTreeMap<String, String> = serde_json::from_str(
        &fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/upstream.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let mut count = 0;
    for (file, source) in corpus {
        let path = Path::new(&file);
        let name = file.rsplit_once('-').unwrap().0;
        macro_rules! check {
            ($t:ty) => {
                roundtrip::<$t>(&source, &path)
            };
        }
        match name {
            "Project" => {
                check!(Project);
                let doc = ProjectDocument::parse(&source).unwrap();
                assert_eq!(doc.source(), source);
            }
            "Reference" => check!(Reference),
            "FileReference" => check!(FileReference),
            "Folder" => check!(Folder),
            "Group" => check!(Group),
            "VariantGroup" => check!(VariantGroup),
            "VersionGroup" => check!(VersionGroup),
            "Target" => check!(Target),
            "ExternalBuildSystemTargetProperties" => check!(ExternalBuildSystemTargetProperties),
            "BuildPhase" => check!(BuildPhase),
            "BuildPhase.Kind" => check!(BuildPhaseKind),
            "BuildPhaseProperties" => check!(BuildPhaseProperties),
            "AppleScriptBuildPhaseProperties" => check!(AppleScriptBuildPhaseProperties),
            "CopyFilesBuildPhaseProperties" => check!(CopyFilesBuildPhaseProperties),
            "ScriptBuildPhaseProperties" => check!(ScriptBuildPhaseProperties),
            "BuildPhaseScope" => check!(BuildPhaseScope),
            "BuildRule" => check!(BuildRule),
            "BuildFileAttributes" => check!(BuildFileAttributes),
            "BuildFileProperties" => check!(BuildFileProperties),
            "BuildFileAttributes.CodeGeneration" => check!(CodeGeneration),
            "BuildFileAttributes.CodeGenerationVisibility" => check!(CodeGenerationVisibility),
            "BuildFileAttributes.HeaderPreservation" => check!(HeaderPreservation),
            "BuildFileAttributes.HeaderRole" => check!(HeaderRole),
            "BuildFileAttributes.MachInterfaceGeneration" => check!(MachInterfaceGeneration),
            "ProjectBuildFile" => check!(ProjectBuildFile),
            "TargetBuildFile" => check!(TargetBuildFile),
            "ProjectBuildPhaseReference" => check!(ProjectBuildPhaseReference),
            "TargetBuildPhaseReference" => check!(TargetBuildPhaseReference),
            "SwiftPackage" => check!(SwiftPackage),
            "SwiftPackageLocation" => check!(SwiftPackageLocation),
            "SwiftPackageVersionConstraint" => check!(SwiftPackageVersionConstraint),
            "SwiftPackageProductReference" => check!(SwiftPackageProductReference),
            "SwiftPackageProductTargetMember" => check!(SwiftPackageProductTargetMember),
            "SwiftPackageProductType" => check!(SwiftPackageProductType),
            "RemoteTarget" => check!(RemoteTarget),
            "RemoteProduct" => check!(RemoteProduct),
            "TargetDependency" => check!(TargetDependency),
            "FolderExceptionSet" => check!(FolderExceptionSet),
            "CommonExceptionSetProperties" => check!(CommonExceptionSetProperties),
            "TargetExceptionSet" => check!(TargetExceptionSet),
            "BuildPhaseExceptionSet" => check!(BuildPhaseExceptionSet),
            "NamePath" => check!(NamePath),
            "NamePathComponent" => check!(NamePathComponent),
            "GroupTreeReference" => check!(GroupTreeReference),
            "GroupTreeAnchoredReference" => check!(GroupTreeAnchoredReference),
            "Configuration" => check!(Configuration),
            "ProjectLocalizationInfo" => check!(ProjectLocalizationInfo),
            "MarketingVersion" => check!(MarketingVersion),
            "MultilineText" => check!(MultilineText),
            "TextEncoding" => check!(TextEncoding),
            "LineEnding" => check!(LineEnding),
            "BuildSetting" => check!(BuildSetting),
            "BundleBasePath" => check!(BundleBasePath),
            "LegacyProvisioningStyle" => check!(LegacyProvisioningStyle),
            "ObjectID" => check!(ObjectId),
            "FileTypeID" => check!(FileTypeId),
            "ProductTypeID" => check!(ProductTypeId),
            "PlatformFilter" => check!(PlatformFilter),
            "Capability" => check!(Capability),
            "AssetTag" => check!(AssetTag),
            "ConfigurationName" => check!(ConfigurationName),
            "Language" => check!(Language),
            "LocalTargetReference" => check!(LocalTargetReference),
            "SwiftPackageName" => check!(SwiftPackageName),
            "FolderMemberID" => check!(FolderMemberId),
            "FilePath" => check!(InlineFilePath),
            "LocalSwiftPackage" => check!(LocalPackage),
            "RemoteSwiftPackage" => check!(RemotePackage),
            "Target.Kind" => check!(TargetKind),
            "Reference.Kind" => check!(ReferenceKind),
            _ => panic!("unhandled fixture {file}"),
        }
        count += 1;
    }
    assert_eq!(
        count, 634,
        "update this count when the upstream corpus changes"
    );
}

// Swift's inline wire wrappers, represented as fields/enum variants in Rust.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct InlineFilePath {
    #[serde(default)]
    path: FilePath,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct LocalPackage {
    path: String,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct RemotePackage {
    repository: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    version: Option<SwiftPackageVersionConstraint>,
}
