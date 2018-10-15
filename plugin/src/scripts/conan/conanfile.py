from conans import ConanFile, CMake
import os
import shutil

def get_requirements():
    L = "${conan.requires}"
    if (L == None or L == ""):
        return None
    return tuple(L.split(","))

class GenericConan(ConanFile):
    name = "${conan.name}"
    version = "${conan.version}"
    url = "none"
    description = "${conan.description}"
    settings = "os", "compiler", "build_type", "arch"
    options = {"shared": [True, False]}
    default_options = "shared=False"
    generators = "cmake"
    exports_sources = ["CMakeLists.txt", "src/*"]
    requires = get_requirements()

    # We do this in build because "source" is only executed by conan once forever.
    def copy_source_files_from(self, module_path):
        dst_dir = os.getcwd()

        print "Copying source from %s to %s" % (module_path, dst_dir)
        for name in os.listdir(module_path):
            srcpath = os.path.join(module_path, name)
            dstpath = os.path.join(dst_dir, name)

            if srcpath.endswith("conanfile.py"):
                continue

            if os.path.exists(dstpath):
                if os.path.isdir(dstpath):
                    shutil.rmtree(dstpath)
                else:
                    os.remove(dstpath)

            if os.path.isfile(srcpath):
                shutil.copy(srcpath, dstpath)
            else:
                shutil.copytree(srcpath, dstpath)

    def generate_filelist(self):
        print "Generating filelist.txt in " + os.getcwd() + " from " + self.source_folder
        f = open("filelist.txt", "w")

        for (path, dirs, files) in os.walk(os.path.join(self.source_folder, "src")):
            for filename in files:
                f.write(os.path.join(path, filename).replace("\\", "/") + "\n")

        f.close()

    def build(self):
        module_path = "${conan.module_path}"

        print "Module: name=%s, version=%s, license=%s" % (self.name, self.version, self.license)
        print "Module Folder: ", module_path
        print "Source Folder: ", self.source_folder
        print "Current Folder: ", os.getcwd()
        print "Requirements:\n", self.requires

        self.copy_source_files_from(module_path)
        self.generate_filelist()

        cmake = CMake(self)
        cmake.configure()
        cmake.build()

        # Explicit way:
        # self.run('cmake %s/hello %s'
        #          % (self.source_folder, cmake.command_line))
        # self.run("cmake --build . %s" % cmake.build_config)

    def package_info(self):
        if "${conan.artifact_type}" == "lib":
            self.cpp_info.libs=["${conan.artifact_name}"]

    def package(self):
        self.copy("*.h", dst="include", src="src")
        self.copy("*.hh", dst="include", src="src")
        self.copy("*.hpp", dst="include", src="src")
        self.copy("*.lib", dst="lib", keep_path=False)
        self.copy("*.dll", dst="bin", keep_path=False)
        self.copy("*.dylib*", dst="lib", keep_path=False)
        self.copy("*.so", dst="lib", keep_path=False, symlinks=True)
        self.copy("*.a", dst="lib", keep_path=False)

