#include <stdio.h>
#include <stddef.h>
#include <string.h>
#include <stdint.h>

#define EXIT_SUCCESS 0
#define EXIT_FAILURE 1

#define HEADER_MAGIC "PXM"
#define FLAT_VERSION 0x10
#define LAYER_VERSION 0x21

void print_info(void) {
    printf("~~~PXM LAYERIZER-INATOR~~~\n"
    "Will convert your non-layered PXM files into layered format without splitting crap up\n"
    "(thanks for forcing my hand, Booster's Lab)\n"
    "Syntax should be: layerizer -i \"in-file\" -o \"out-file\"\n"
    );
}

int return_failure(char* reason) {
    printf("Err: %s\n\n", reason);
    print_info();
    return EXIT_FAILURE;
}


unsigned short read_le16(FILE *stream)
{
	unsigned char bytes[2];

	fread(bytes, 2, 1, stream);

	return (bytes[1] << 8) | bytes[0];
}


int main (int argc, char *argv[]) {

    char* infilepath = NULL;
    char* outfilepath = NULL;

    //parse input
    {

        for(int i = 0; i < argc; ++i) {
            char* argument = argv[i];

            //if we get a command
            if(argument[0] == '-') {
                switch(argument[1]) {
                    case 'i': {
                        //no arguments after, fail out
                        if(i == argc - 1) {
                            return return_failure((char*)"could not parse command!");
                        }
                        //next argument must be the infile path
                        infilepath = argv[++i];
                        break; //to later args
                    }
                    case 'o': {
                        //no arguments after, fail out
                        if(i == argc - 1) {
                            return return_failure((char*)"could not parse command!");
                        }
                        //next argument must be the outfile path
                        outfilepath = argv[++i];
                        break;
                    }

                    //invalid - assume help
                    default: {
                        return return_failure((char*)"could not parse command!");
                    }
                }
            }


        }
    }

    if(infilepath == NULL || outfilepath == NULL){
        return return_failure((char*)"could not parse command!");
    }


    printf("in-file: %s\n", infilepath);
    printf("out-file: %s\n", outfilepath);

    FILE* ifp = fopen(infilepath, "rb");
    FILE* ofp = fopen(outfilepath, "wb");

    if(ifp == NULL || ofp == NULL){
        return return_failure((char*)"invalid in-file or out-file!");
    }

    //read and check header
    char in_magic[3] = {0};
    fread(in_magic, sizeof(char), 3, ifp);

    if(strcmp(in_magic, HEADER_MAGIC)) {
        return_failure((char*)"invalid in-file type! header is not PXM!");
    }

    //version header (must be 0x10)
    if(FLAT_VERSION != fgetc(ifp)) {
        return_failure((char*)"PXM version is incorrect, has it already been layerized?");
    }

    //get width and height amalgamated (4 bytes total)
    uint32_t dimensions;
    fread(&dimensions, sizeof(uint32_t), 1, ifp);


    //set up new file header
    fwrite((char*)HEADER_MAGIC, sizeof(char), 3, ofp);
    fputc(LAYER_VERSION, ofp);
    fwrite(&dimensions, sizeof(uint32_t), 1, ofp);


    //do a per-byte process of the file
    uint32_t buff_count = 0;
    while (1) {
        int c = fgetc(ifp);
        //EOF
        if(c < 0) {
            break;
        }

        //force little-endian cast of number
        char outs[2] = {
            (char)c,
            0
        };

        fwrite(outs, 2, 1, ofp);

        ++buff_count;
    }

    //fill out rest of layers (3 more layers with 2 bytes per attribute)
    for(uint32_t i = 0; i < buff_count * 3 * 2; ++i) {
        fputc(0, ofp);
    }

    fclose(ifp);
    fclose(ofp);

    printf("Finished.\n");

    return 0;
}


